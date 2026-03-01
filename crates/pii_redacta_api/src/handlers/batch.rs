//! Batch Processing handler
//!
//! Sprint 14: Submit, poll, and retrieve results for batch PII detection jobs.

use crate::extractors::AuthUser;
use crate::AppState;
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use pii_redacta_core::detection::custom::{CustomRule, CustomRuleDetector};
use pii_redacta_core::detection::PatternDetector;
use pii_redacta_core::tokenization::Tokenizer;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// Request / Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchRequest {
    pub items: Vec<String>,
    pub redact: Option<bool>,
    pub use_custom_rules: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchResponse {
    pub batch_id: String,
    pub status: String,
    pub total_items: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchStatusResponse {
    pub id: String,
    pub status: String,
    pub total_items: i32,
    pub completed_items: i32,
    pub failed_items: i32,
    pub redact: bool,
    pub use_custom_rules: bool,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchResultItem {
    pub item_index: i32,
    pub status: String,
    pub entities: Option<serde_json::Value>,
    pub redacted_text: Option<String>,
    pub processing_time_ms: Option<i32>,
    pub error_message: Option<String>,
}

// ============================================================================
// Error Type
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum BatchError {
    #[error("Batch processing is not available on your current plan")]
    NotAvailable,
    #[error("Batch item limit exceeded (max {0} items)")]
    LimitExceeded(i32),
    #[error("Batch job not found")]
    NotFound,
    #[error("Items list cannot be empty")]
    EmptyItems,
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for BatchError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            BatchError::NotAvailable => (StatusCode::FORBIDDEN, self.to_string()),
            BatchError::LimitExceeded(_) => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
            BatchError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            BatchError::EmptyItems => (StatusCode::BAD_REQUEST, self.to_string()),
            BatchError::Database(ref e) => {
                tracing::error!("Batch database error: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An unexpected error occurred".to_string(),
                )
            }
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
// Tier Gating Helper
// ============================================================================

async fn check_batch_access(
    state: &AppState,
    user_id: Uuid,
) -> Result<pii_redacta_core::db::models::TierLimits, BatchError> {
    let row = sqlx::query_as::<_, (serde_json::Value, serde_json::Value)>(
        r#"
        SELECT t.limits, t.features
        FROM subscriptions s
        JOIN tiers t ON s.tier_id = t.id
        JOIN users u ON u.id = s.user_id
        WHERE s.user_id = $1
        AND u.deleted_at IS NULL
        AND s.status IN ('trial', 'active', 'past_due')
        AND (s.current_period_end IS NULL OR s.current_period_end > NOW())
        ORDER BY s.created_at DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(state.db.pool())
    .await?;

    let (limits_json, features_json) = row.ok_or(BatchError::NotAvailable)?;

    let features: pii_redacta_core::db::models::TierFeatures =
        serde_json::from_value(features_json).unwrap_or_default();
    if !features.batch_processing {
        return Err(BatchError::NotAvailable);
    }

    let limits: pii_redacta_core::db::models::TierLimits =
        serde_json::from_value(limits_json).unwrap_or_default();

    Ok(limits)
}

// ============================================================================
// Handlers
// ============================================================================

/// POST /api/v1/batch/detect — Submit a batch detection job
pub async fn submit_batch(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<BatchRequest>,
) -> Result<(StatusCode, Json<BatchResponse>), BatchError> {
    let limits = check_batch_access(&state, auth_user.user_id).await?;

    if request.items.is_empty() {
        return Err(BatchError::EmptyItems);
    }

    // Check item count limit
    if let Some(max_items) = limits.max_batch_items {
        if request.items.len() > max_items as usize {
            return Err(BatchError::LimitExceeded(max_items));
        }
    }

    let total_items = request.items.len() as i32;
    let redact = request.redact.unwrap_or(false);
    let use_custom_rules = request.use_custom_rules.unwrap_or(false);

    // Insert batch job
    let batch_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO batch_jobs (user_id, total_items, redact, use_custom_rules)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(auth_user.user_id)
    .bind(total_items)
    .bind(redact)
    .bind(use_custom_rules)
    .fetch_one(state.db.pool())
    .await?;

    // Insert batch items
    for (index, text) in request.items.iter().enumerate() {
        sqlx::query(
            r#"
            INSERT INTO batch_items (batch_id, item_index, input_text)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(batch_id)
        .bind(index as i32)
        .bind(text)
        .execute(state.db.pool())
        .await?;
    }

    // Spawn background processing task
    let db = state.db.clone();
    let user_id = auth_user.user_id;
    tokio::spawn(async move {
        process_batch(db, batch_id, user_id, redact, use_custom_rules).await;
    });

    Ok((
        StatusCode::CREATED,
        Json(BatchResponse {
            batch_id: batch_id.to_string(),
            status: "pending".to_string(),
            total_items,
        }),
    ))
}

/// GET /api/v1/batch/:batch_id — Poll batch status
pub async fn get_batch_status(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(batch_id): Path<Uuid>,
) -> Result<Json<BatchStatusResponse>, BatchError> {
    let row = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            i32,
            i32,
            i32,
            bool,
            bool,
            chrono::DateTime<chrono::Utc>,
            Option<chrono::DateTime<chrono::Utc>>,
        ),
    >(
        r#"
        SELECT id, status, total_items, completed_items, failed_items,
               redact, use_custom_rules, created_at, completed_at
        FROM batch_jobs
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(batch_id)
    .bind(auth_user.user_id)
    .fetch_optional(state.db.pool())
    .await?
    .ok_or(BatchError::NotFound)?;

    Ok(Json(BatchStatusResponse {
        id: row.0.to_string(),
        status: row.1,
        total_items: row.2,
        completed_items: row.3,
        failed_items: row.4,
        redact: row.5,
        use_custom_rules: row.6,
        created_at: row.7.to_rfc3339(),
        completed_at: row.8.map(|t| t.to_rfc3339()),
    }))
}

/// GET /api/v1/batch/:batch_id/results — Get batch results
pub async fn get_batch_results(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(batch_id): Path<Uuid>,
) -> Result<Json<Vec<BatchResultItem>>, BatchError> {
    // Verify ownership
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM batch_jobs WHERE id = $1 AND user_id = $2)",
    )
    .bind(batch_id)
    .bind(auth_user.user_id)
    .fetch_one(state.db.pool())
    .await?;

    if !exists {
        return Err(BatchError::NotFound);
    }

    let rows = sqlx::query_as::<
        _,
        (
            i32,
            String,
            Option<serde_json::Value>,
            Option<String>,
            Option<i32>,
            Option<String>,
        ),
    >(
        r#"
        SELECT item_index, status, entities, redacted_text,
               processing_time_ms, error_message
        FROM batch_items
        WHERE batch_id = $1
        ORDER BY item_index
        "#,
    )
    .bind(batch_id)
    .fetch_all(state.db.pool())
    .await?;

    let items: Vec<BatchResultItem> = rows
        .into_iter()
        .map(|r| BatchResultItem {
            item_index: r.0,
            status: r.1,
            entities: r.2,
            redacted_text: r.3,
            processing_time_ms: r.4,
            error_message: r.5,
        })
        .collect();

    Ok(Json(items))
}

// ============================================================================
// Background Processing
// ============================================================================

async fn process_batch(
    db: Arc<pii_redacta_core::db::Database>,
    batch_id: Uuid,
    user_id: Uuid,
    redact: bool,
    use_custom_rules: bool,
) {
    // Update status to processing
    if let Err(e) =
        sqlx::query("UPDATE batch_jobs SET status = 'processing', updated_at = NOW() WHERE id = $1")
            .bind(batch_id)
            .execute(db.pool())
            .await
    {
        tracing::error!(batch_id = %batch_id, error = %e, "Failed to update batch status");
        return;
    }

    // Load custom rules if requested
    let custom_rules: Vec<CustomRule> = if use_custom_rules {
        match sqlx::query_as::<_, (Uuid, String, String, String, f32)>(
            r#"
            SELECT id, name, pattern, entity_label, confidence
            FROM custom_rules
            WHERE user_id = $1 AND is_active = true
            "#,
        )
        .bind(user_id)
        .fetch_all(db.pool())
        .await
        {
            Ok(rows) => rows
                .into_iter()
                .map(|r| CustomRule {
                    id: r.0,
                    name: r.1,
                    pattern: r.2,
                    entity_label: r.3,
                    confidence: r.4,
                })
                .collect(),
            Err(e) => {
                tracing::error!(batch_id = %batch_id, error = %e, "Failed to load custom rules");
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    // Load batch items
    let items: Vec<(Uuid, i32, String)> = match sqlx::query_as(
        r#"
        SELECT id, item_index, input_text
        FROM batch_items
        WHERE batch_id = $1
        ORDER BY item_index
        "#,
    )
    .bind(batch_id)
    .fetch_all(db.pool())
    .await
    {
        Ok(items) => items,
        Err(e) => {
            tracing::error!(batch_id = %batch_id, error = %e, "Failed to load batch items");
            let _ = sqlx::query(
                "UPDATE batch_jobs SET status = 'failed', error_message = $1, completed_at = NOW(), updated_at = NOW() WHERE id = $2",
            )
            .bind("Failed to load batch items")
            .bind(batch_id)
            .execute(db.pool())
            .await;
            return;
        }
    };

    let detector = PatternDetector::new();
    let custom_detector = CustomRuleDetector::new();
    let mut completed = 0i32;
    let mut failed = 0i32;

    for (item_id, _item_index, input_text) in &items {
        let start = std::time::Instant::now();

        // Detect PII
        let mut entities = detector.detect_all(input_text);

        // Run custom rules
        if use_custom_rules && !custom_rules.is_empty() {
            let custom_entities = custom_detector.detect(input_text, &custom_rules);
            entities.extend(custom_entities);
        }

        let processing_time_ms = i32::try_from(start.elapsed().as_millis()).unwrap_or(i32::MAX);

        // Optional redaction
        let redacted_text = if redact && !entities.is_empty() {
            let tenant_id = user_id.to_string();
            let tokenizer = Tokenizer::new(&tenant_id);
            let (tokenized, _) = tokenizer.tokenize(input_text, &entities);
            Some(tokenized)
        } else {
            None
        };

        let entities_json = serde_json::to_value(&entities).ok();

        // Update item
        match sqlx::query(
            r#"
            UPDATE batch_items
            SET status = 'completed', entities = $1, redacted_text = $2,
                processing_time_ms = $3, completed_at = NOW()
            WHERE id = $4
            "#,
        )
        .bind(&entities_json)
        .bind(&redacted_text)
        .bind(processing_time_ms)
        .bind(item_id)
        .execute(db.pool())
        .await
        {
            Ok(_) => completed += 1,
            Err(e) => {
                tracing::warn!(item_id = %item_id, error = %e, "Failed to update batch item");
                let err_msg: String = e.to_string().chars().take(500).collect();
                let _ = sqlx::query(
                    r#"
                    UPDATE batch_items
                    SET status = 'failed', error_message = $1, completed_at = NOW()
                    WHERE id = $2
                    "#,
                )
                .bind(&err_msg)
                .bind(item_id)
                .execute(db.pool())
                .await;
                failed += 1;
            }
        }

        // Update progress
        let _ = sqlx::query(
            "UPDATE batch_jobs SET completed_items = $1, failed_items = $2, updated_at = NOW() WHERE id = $3",
        )
        .bind(completed)
        .bind(failed)
        .bind(batch_id)
        .execute(db.pool())
        .await;
    }

    // Final status
    let status = if failed == 0 {
        "completed"
    } else if completed == 0 {
        "failed"
    } else {
        "partial"
    };

    let request_type = if redact {
        "batch_redact"
    } else {
        "batch_detect"
    };

    let _ = sqlx::query(
        r#"
        UPDATE batch_jobs
        SET status = $1, completed_at = NOW(), updated_at = NOW()
        WHERE id = $2
        "#,
    )
    .bind(status)
    .bind(batch_id)
    .execute(db.pool())
    .await;

    // Record usage
    let _ = sqlx::query(
        r#"
        INSERT INTO usage_logs (user_id, request_type, processing_time_ms, detections_count, success, created_at)
        VALUES ($1, $2, NULL, $3, $4, NOW())
        "#,
    )
    .bind(user_id)
    .bind(request_type)
    .bind(completed)
    .bind(status != "failed")
    .execute(db.pool())
    .await;

    tracing::info!(
        batch_id = %batch_id,
        status = status,
        completed = completed,
        failed = failed,
        "Batch processing finished"
    );
}

#[cfg(test)]
#[path = "batch_test.rs"]
mod tests;
