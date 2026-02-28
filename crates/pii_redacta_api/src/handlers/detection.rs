//! Detection handler

use crate::extractors::AuthUser;
use crate::AppState;
use axum::{
    extract::{ConnectInfo, Extension, State},
    http::StatusCode,
    Json,
};
use pii_redacta_core::detection::PatternDetector;
use pii_redacta_core::tokenization::Tokenizer;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;

use super::JobQueue;

/// Maximum text length (1 MB) to prevent excessive resource usage
const MAX_TEXT_LENGTH: usize = 1_000_000;

#[derive(Deserialize)]
pub struct DetectRequest {
    pub text: String,
    pub options: Option<DetectionOptions>,
}

#[derive(Deserialize)]
pub struct DetectionOptions {
    pub redact: bool,
    pub tenant_id: Option<String>,
}

#[derive(Serialize)]
pub struct DetectResponse {
    pub entities: Vec<pii_redacta_core::types::Entity>,
    pub processing_time_ms: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redacted_text: Option<String>,
}

pub async fn detect(
    State(_queue): State<Arc<JobQueue>>,
    Json(request): Json<DetectRequest>,
) -> (StatusCode, Json<DetectResponse>) {
    // Validate input
    if request.text.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(DetectResponse {
                entities: Vec::new(),
                processing_time_ms: 0.0,
                redacted_text: None,
            }),
        );
    }

    if request.text.len() > MAX_TEXT_LENGTH {
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(DetectResponse {
                entities: Vec::new(),
                processing_time_ms: 0.0,
                redacted_text: None,
            }),
        );
    }

    let detector = PatternDetector::new();
    let start = std::time::Instant::now();

    // Detect entities
    let entities = detector.detect_all(&request.text);
    let processing_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    // Handle redaction if requested
    let redacted_text = if let Some(options) = request.options {
        if options.redact && !entities.is_empty() {
            let tenant_id = options.tenant_id.as_deref().unwrap_or("default");
            let tokenizer = Tokenizer::new(tenant_id);
            let (tokenized, _token_map) = tokenizer.tokenize(&request.text, &entities);
            Some(tokenized)
        } else {
            None
        }
    } else {
        None
    };

    (
        StatusCode::OK,
        Json(DetectResponse {
            entities,
            processing_time_ms,
            redacted_text,
        }),
    )
}

/// Authenticated detection handler — same logic as `detect()` but records usage.
pub async fn detect_authenticated(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(request): Json<DetectRequest>,
) -> (StatusCode, Json<DetectResponse>) {
    if request.text.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(DetectResponse {
                entities: Vec::new(),
                processing_time_ms: 0.0,
                redacted_text: None,
            }),
        );
    }

    if request.text.len() > MAX_TEXT_LENGTH {
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(DetectResponse {
                entities: Vec::new(),
                processing_time_ms: 0.0,
                redacted_text: None,
            }),
        );
    }

    let detector = PatternDetector::new();
    let start = std::time::Instant::now();

    let entities = detector.detect_all(&request.text);
    let processing_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    let redacted_text = if let Some(options) = request.options {
        if options.redact && !entities.is_empty() {
            let tenant_id = options.tenant_id.as_deref().unwrap_or("default");
            let tokenizer = Tokenizer::new(tenant_id);
            let (tokenized, _token_map) = tokenizer.tokenize(&request.text, &entities);
            Some(tokenized)
        } else {
            None
        }
    } else {
        None
    };

    let detections_count = entities.len() as i32;
    let response = DetectResponse {
        entities,
        processing_time_ms,
        redacted_text,
    };

    // Fire-and-forget usage recording
    let pool = state.db.pool().clone();
    let user_id = auth_user.user_id;
    let proc_ms = processing_time_ms as i32;
    let ip = connect_info.map(|ci| ci.0.ip().to_string());
    tokio::spawn(async move {
        let record = pii_redacta_core::db::usage::UsageRecord {
            user_id,
            api_key_id: None,
            request_type: "api_detect",
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
            tracing::warn!("Failed to record usage: {e}");
        }
    });

    (StatusCode::OK, Json(response))
}

#[cfg(test)]
#[path = "detection_test.rs"]
mod tests;
