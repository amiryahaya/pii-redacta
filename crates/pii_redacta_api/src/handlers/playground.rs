//! Playground handlers for interactive PII detection
//!
//! Sprint 13: Authenticated Playground — text and file analysis with tier-based daily quotas.

use crate::extractors::AuthUser;
use crate::AppState;
use axum::{
    body::Body,
    extract::{ConnectInfo, Extension, State},
    http::{Request, StatusCode},
    response::IntoResponse,
    Json,
};
use pii_redacta_core::detection::PatternDetector;
use pii_redacta_core::extraction::Extractor;
use pii_redacta_core::tokenization::Tokenizer;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use uuid::Uuid;

/// Maximum text length (1 MB)
const MAX_TEXT_LENGTH: usize = 1_000_000;

// ============================================================================
// Request / Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct PlaygroundTextRequest {
    pub text: String,
    pub redact: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaygroundResponse {
    pub entities: Vec<pii_redacta_core::types::Entity>,
    pub processing_time_ms: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redacted_text: Option<String>,
    pub text_length: usize,
    pub daily_usage: PlaygroundUsage,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaygroundUsage {
    pub used_today: i64,
    pub daily_limit: Option<i32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaygroundHistoryEntry {
    pub id: String,
    pub request_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_type: Option<String>,
    pub detections_count: Option<i32>,
    pub processing_time_ms: Option<i32>,
    pub success: bool,
    pub created_at: String,
}

// ============================================================================
// Error Type
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum PlaygroundError {
    #[error("Playground is not available on your current plan")]
    NotAvailable,
    #[error("Daily playground limit reached")]
    DailyLimitReached,
    #[error("File too large")]
    FileTooLarge,
    #[error("Text too long (max 1MB)")]
    TextTooLong,
    #[error("Input text is empty")]
    EmptyInput,
    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),
    #[error("Text extraction failed: {0}")]
    ExtractionFailed(String),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for PlaygroundError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            PlaygroundError::NotAvailable => (StatusCode::FORBIDDEN, self.to_string()),
            PlaygroundError::DailyLimitReached => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
            PlaygroundError::FileTooLarge => (StatusCode::PAYLOAD_TOO_LARGE, self.to_string()),
            PlaygroundError::TextTooLong => (StatusCode::PAYLOAD_TOO_LARGE, self.to_string()),
            PlaygroundError::EmptyInput => (StatusCode::BAD_REQUEST, self.to_string()),
            PlaygroundError::UnsupportedFileType(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            PlaygroundError::ExtractionFailed(_) => {
                (StatusCode::UNPROCESSABLE_ENTITY, self.to_string())
            }
            PlaygroundError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unexpected error occurred".to_string(),
            ),
        };
        let body = serde_json::json!({
            "error": {
                "code": status.as_u16(),
                "message": message,
            }
        });
        (status, Json(body)).into_response()
    }
}

// ============================================================================
// Quota Helper
// ============================================================================

struct PlaygroundQuota {
    daily_limit: Option<i32>,
    max_file_size: Option<i64>,
    used_today: i64,
}

async fn check_playground_quota(
    state: &AppState,
    user_id: Uuid,
) -> Result<PlaygroundQuota, PlaygroundError> {
    // Look up the user's subscription → tier limits/features
    let row = sqlx::query_as::<_, (serde_json::Value, serde_json::Value)>(
        r#"
        SELECT t.limits, t.features
        FROM subscriptions s
        JOIN tiers t ON s.tier_id = t.id
        WHERE s.user_id = $1
        AND s.status IN ('trial', 'active', 'past_due')
        ORDER BY s.created_at DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(state.db.pool())
    .await?;

    let (limits_json, features_json) = row.ok_or(PlaygroundError::NotAvailable)?;

    let features: pii_redacta_core::db::models::TierFeatures =
        serde_json::from_value(features_json).unwrap_or_default();
    if !features.playground {
        return Err(PlaygroundError::NotAvailable);
    }

    let limits: pii_redacta_core::db::models::TierLimits =
        serde_json::from_value(limits_json).unwrap_or_default();

    // Count today's playground usage
    let used_today: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM usage_logs
        WHERE user_id = $1
        AND request_type IN ('playground', 'playground_file')
        AND created_at >= CURRENT_DATE
        AND success = true
        "#,
    )
    .bind(user_id)
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    // Check daily limit
    if let Some(max_daily) = limits.playground_max_daily {
        if used_today >= max_daily as i64 {
            return Err(PlaygroundError::DailyLimitReached);
        }
    }

    Ok(PlaygroundQuota {
        daily_limit: limits.playground_max_daily,
        max_file_size: limits.playground_max_file_size,
        used_today,
    })
}

// ============================================================================
// Handlers
// ============================================================================

/// POST /api/v1/playground/text — Detect PII in pasted text
pub async fn playground_text(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(request): Json<PlaygroundTextRequest>,
) -> Result<Json<PlaygroundResponse>, PlaygroundError> {
    // Validate input
    if request.text.is_empty() {
        return Err(PlaygroundError::EmptyInput);
    }
    if request.text.len() > MAX_TEXT_LENGTH {
        return Err(PlaygroundError::TextTooLong);
    }

    // Quota check
    let quota = check_playground_quota(&state, auth_user.user_id).await?;

    // Detect PII
    let detector = PatternDetector::new();
    let start = std::time::Instant::now();
    let entities = detector.detect_all(&request.text);
    let processing_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    // Optional redaction
    let redacted_text = if request.redact.unwrap_or(false) && !entities.is_empty() {
        let tokenizer = Tokenizer::new("playground");
        let (tokenized, _) = tokenizer.tokenize(&request.text, &entities);
        Some(tokenized)
    } else {
        None
    };

    let text_length = request.text.len();
    let detections_count = entities.len() as i32;

    // Record metrics
    state
        .metrics
        .record_detection(detections_count as u64, processing_time_ms);

    // Fire-and-forget usage recording
    let pool = state.db.pool().clone();
    let user_id = auth_user.user_id;
    let proc_ms = processing_time_ms as i32;
    let ip = connect_info.map(|ci| ci.0.ip().to_string());
    tokio::spawn(async move {
        let record = pii_redacta_core::db::usage::UsageRecord {
            user_id,
            api_key_id: None,
            request_type: "playground",
            file_name: None,
            file_size_bytes: None,
            file_type: None,
            processing_time_ms: Some(proc_ms),
            page_count: None,
            detections_count: Some(detections_count),
            success: true,
            error_message: None,
            ip_address: ip.as_deref(),
        };
        if let Err(e) = pii_redacta_core::db::usage::record_usage(&pool, &record).await {
            tracing::warn!("Failed to record playground usage: {e}");
        }
    });

    Ok(Json(PlaygroundResponse {
        entities,
        processing_time_ms,
        redacted_text,
        text_length,
        daily_usage: PlaygroundUsage {
            used_today: quota.used_today + 1,
            daily_limit: quota.daily_limit,
        },
    }))
}

/// POST /api/v1/playground/file — Detect PII in uploaded file
pub async fn playground_file(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    request: Request<Body>,
) -> Result<Json<PlaygroundResponse>, PlaygroundError> {
    // Quota check
    let quota = check_playground_quota(&state, auth_user.user_id).await?;

    // Parse multipart
    let (file_bytes, file_name, mime_type) = parse_playground_upload(request).await?;

    // Check file size against tier limit
    if let Some(max_size) = quota.max_file_size {
        if file_bytes.len() as i64 > max_size {
            return Err(PlaygroundError::FileTooLarge);
        }
    }

    // Extract text
    let extracted = Extractor::extract(&file_bytes, Some(&mime_type))
        .map_err(|e| PlaygroundError::ExtractionFailed(e.to_string()))?;

    if extracted.text.is_empty() {
        return Err(PlaygroundError::ExtractionFailed(
            "No text could be extracted from the file".to_string(),
        ));
    }

    // Detect PII
    let detector = PatternDetector::new();
    let start = std::time::Instant::now();
    let entities = detector.detect_all(&extracted.text);
    let processing_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    // Optional redaction (check query param via file name convention, default false)
    let redacted_text = if !entities.is_empty() {
        let tokenizer = Tokenizer::new("playground");
        let (tokenized, _) = tokenizer.tokenize(&extracted.text, &entities);
        Some(tokenized)
    } else {
        None
    };

    let text_length = extracted.text.len();
    let detections_count = entities.len() as i32;
    let file_size = file_bytes.len() as i32;

    // Record metrics
    state
        .metrics
        .record_detection(detections_count as u64, processing_time_ms);

    // Fire-and-forget usage recording
    let pool = state.db.pool().clone();
    let user_id = auth_user.user_id;
    let proc_ms = processing_time_ms as i32;
    let ip = connect_info.map(|ci| ci.0.ip().to_string());
    let fname = file_name.clone();
    let ftype = mime_type.clone();
    tokio::spawn(async move {
        let record = pii_redacta_core::db::usage::UsageRecord {
            user_id,
            api_key_id: None,
            request_type: "playground_file",
            file_name: fname.as_deref(),
            file_size_bytes: Some(file_size),
            file_type: Some(&ftype),
            processing_time_ms: Some(proc_ms),
            page_count: None,
            detections_count: Some(detections_count),
            success: true,
            error_message: None,
            ip_address: ip.as_deref(),
        };
        if let Err(e) = pii_redacta_core::db::usage::record_usage(&pool, &record).await {
            tracing::warn!("Failed to record playground file usage: {e}");
        }
    });

    Ok(Json(PlaygroundResponse {
        entities,
        processing_time_ms,
        redacted_text,
        text_length,
        daily_usage: PlaygroundUsage {
            used_today: quota.used_today + 1,
            daily_limit: quota.daily_limit,
        },
    }))
}

/// GET /api/v1/playground/history — Last 20 playground submissions
pub async fn playground_history(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Vec<PlaygroundHistoryEntry>>, PlaygroundError> {
    let rows = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            Option<String>,
            Option<String>,
            Option<i32>,
            Option<i32>,
            bool,
            chrono::DateTime<chrono::Utc>,
        ),
    >(
        r#"
        SELECT
            id, request_type, file_name, file_type,
            detections_count, processing_time_ms,
            success, created_at
        FROM usage_logs
        WHERE user_id = $1
        AND request_type IN ('playground', 'playground_file')
        ORDER BY created_at DESC
        LIMIT 20
        "#,
    )
    .bind(auth_user.user_id)
    .fetch_all(state.db.pool())
    .await?;

    let entries: Vec<PlaygroundHistoryEntry> = rows
        .into_iter()
        .map(|row| PlaygroundHistoryEntry {
            id: row.0.to_string(),
            request_type: row.1,
            file_name: row.2,
            file_type: row.3,
            detections_count: row.4,
            processing_time_ms: row.5,
            success: row.6,
            created_at: row.7.to_rfc3339(),
        })
        .collect();

    Ok(Json(entries))
}

// ============================================================================
// Multipart Parser (Playground)
// ============================================================================

/// Supported MIME types for playground file upload
const SUPPORTED_MIME_TYPES: &[&str] = &[
    "text/plain",
    "text/csv",
    "application/pdf",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    "application/vnd.openxmlformats",
];

/// Parse multipart form data for playground file upload.
/// Returns (file_bytes, file_name, mime_type).
async fn parse_playground_upload(
    request: Request<Body>,
) -> Result<(Vec<u8>, Option<String>, String), PlaygroundError> {
    let content_type = request
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    if !content_type.starts_with("multipart/form-data") {
        return Err(PlaygroundError::UnsupportedFileType(
            "Expected multipart/form-data".to_string(),
        ));
    }

    let boundary = content_type
        .split("boundary=")
        .nth(1)
        .ok_or_else(|| PlaygroundError::UnsupportedFileType("Missing boundary".to_string()))?
        .to_string();

    let body_bytes = axum::body::to_bytes(
        request.into_body(),
        crate::middleware::security::DEFAULT_MAX_BODY_SIZE,
    )
    .await
    .map_err(|_| PlaygroundError::FileTooLarge)?;

    let body_str = String::from_utf8_lossy(&body_bytes);
    let boundary_str = format!("--{}", boundary);

    let mut file_content = Vec::new();
    let mut mime_type = "application/octet-stream".to_string();
    let mut file_name: Option<String> = None;
    let mut found_file = false;

    for part in body_str.split(&boundary_str) {
        if part.contains("Content-Disposition: form-data") && part.contains("name=\"file\"") {
            found_file = true;

            // Extract filename
            if let Some(fn_start) = part.find("filename=\"") {
                let after = &part[fn_start + 10..];
                if let Some(fn_end) = after.find('"') {
                    let name = after[..fn_end].to_string();
                    if !name.is_empty() {
                        file_name = Some(name);
                    }
                }
            }

            // Extract content-type
            if let Some(ct_line) = part.lines().find(|l| l.starts_with("Content-Type:")) {
                mime_type = ct_line
                    .split(':')
                    .nth(1)
                    .unwrap_or("application/octet-stream")
                    .trim()
                    .to_string();
            }

            // Extract body
            if let Some(content_start) = part.find("\r\n\r\n") {
                let content = &part[content_start + 4..];
                let content = content.trim_end_matches("\r\n");
                file_content = content.as_bytes().to_vec();
            }
            break;
        }
    }

    if !found_file || file_content.is_empty() {
        return Err(PlaygroundError::EmptyInput);
    }

    // Validate MIME type
    if !SUPPORTED_MIME_TYPES.contains(&mime_type.as_str()) {
        return Err(PlaygroundError::UnsupportedFileType(mime_type));
    }

    Ok((file_content, file_name, mime_type))
}

#[cfg(test)]
#[path = "playground_test.rs"]
mod tests;
