//! Custom Rules handler
//!
//! Sprint 14: CRUD + test endpoints for user-defined detection rules.

use crate::extractors::AuthUser;
use crate::AppState;
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use pii_redacta_core::detection::custom::{CustomRule, CustomRuleDetector};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Request / Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRuleRequest {
    pub name: String,
    pub description: Option<String>,
    pub pattern: String,
    pub entity_label: String,
    pub confidence: Option<f32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRuleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub pattern: Option<String>,
    pub entity_label: Option<String>,
    pub confidence: Option<f32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestRuleRequest {
    pub text: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub pattern: String,
    pub entity_label: String,
    pub confidence: f32,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleTestResponse {
    pub matches: Vec<RuleTestMatch>,
    pub processing_time_ms: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleTestMatch {
    pub value: String,
    pub start: usize,
    pub end: usize,
    pub entity_label: String,
    pub confidence: f32,
}

// ============================================================================
// Error Type
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum RuleError {
    #[error("Custom rules are not available on your current plan")]
    NotAvailable,
    #[error("Custom rule limit reached for your plan")]
    LimitReached,
    #[error("Invalid regex pattern: {0}")]
    InvalidPattern(String),
    #[error("Rule not found")]
    NotFound,
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for RuleError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            RuleError::NotAvailable => (StatusCode::FORBIDDEN, self.to_string()),
            RuleError::LimitReached => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
            RuleError::InvalidPattern(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            RuleError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            RuleError::InvalidInput(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            RuleError::Database(ref e) => {
                tracing::error!("Rules database error: {e}");
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

async fn check_custom_rules_access(
    state: &AppState,
    user_id: Uuid,
) -> Result<pii_redacta_core::db::models::TierLimits, RuleError> {
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

    let (limits_json, features_json) = row.ok_or(RuleError::NotAvailable)?;

    let features: pii_redacta_core::db::models::TierFeatures =
        serde_json::from_value(features_json).unwrap_or_default();
    if !features.custom_rules {
        return Err(RuleError::NotAvailable);
    }

    let limits: pii_redacta_core::db::models::TierLimits =
        serde_json::from_value(limits_json).unwrap_or_default();

    Ok(limits)
}

// ============================================================================
// Handlers
// ============================================================================

/// POST /api/v1/rules — Create a custom rule
pub async fn create_rule(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<CreateRuleRequest>,
) -> Result<(StatusCode, Json<RuleResponse>), RuleError> {
    let limits = check_custom_rules_access(&state, auth_user.user_id).await?;

    // Validate inputs
    if request.name.is_empty() || request.name.len() > 100 {
        return Err(RuleError::InvalidInput(
            "Name must be 1-100 characters".to_string(),
        ));
    }
    if request.entity_label.is_empty() || request.entity_label.len() > 50 {
        return Err(RuleError::InvalidInput(
            "Entity label must be 1-50 characters".to_string(),
        ));
    }

    let confidence = request.confidence.unwrap_or(0.9);
    if !(0.0..=1.0).contains(&confidence) {
        return Err(RuleError::InvalidInput(
            "Confidence must be between 0.0 and 1.0".to_string(),
        ));
    }

    // Validate regex pattern
    CustomRuleDetector::validate_pattern(&request.pattern).map_err(RuleError::InvalidPattern)?;

    // Check rule count limit
    if let Some(max_rules) = limits.max_custom_rules {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM custom_rules WHERE user_id = $1")
            .bind(auth_user.user_id)
            .fetch_one(state.db.pool())
            .await?;

        if count >= max_rules as i64 {
            return Err(RuleError::LimitReached);
        }
    }

    // Insert rule
    let row = sqlx::query_as::<
        _,
        (
            Uuid,
            chrono::DateTime<chrono::Utc>,
            chrono::DateTime<chrono::Utc>,
        ),
    >(
        r#"
        INSERT INTO custom_rules (user_id, name, description, pattern, entity_label, confidence)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, created_at, updated_at
        "#,
    )
    .bind(auth_user.user_id)
    .bind(&request.name)
    .bind(&request.description)
    .bind(&request.pattern)
    .bind(&request.entity_label)
    .bind(confidence)
    .fetch_one(state.db.pool())
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(RuleResponse {
            id: row.0.to_string(),
            name: request.name,
            description: request.description,
            pattern: request.pattern,
            entity_label: request.entity_label,
            confidence,
            is_active: true,
            created_at: row.1.to_rfc3339(),
            updated_at: row.2.to_rfc3339(),
        }),
    ))
}

/// GET /api/v1/rules — List user's custom rules
pub async fn list_rules(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Vec<RuleResponse>>, RuleError> {
    check_custom_rules_access(&state, auth_user.user_id).await?;

    let rows = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            Option<String>,
            String,
            String,
            f32,
            bool,
            chrono::DateTime<chrono::Utc>,
            chrono::DateTime<chrono::Utc>,
        ),
    >(
        r#"
        SELECT id, name, description, pattern, entity_label, confidence,
               is_active, created_at, updated_at
        FROM custom_rules
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(auth_user.user_id)
    .fetch_all(state.db.pool())
    .await?;

    let rules: Vec<RuleResponse> = rows
        .into_iter()
        .map(|r| RuleResponse {
            id: r.0.to_string(),
            name: r.1,
            description: r.2,
            pattern: r.3,
            entity_label: r.4,
            confidence: r.5,
            is_active: r.6,
            created_at: r.7.to_rfc3339(),
            updated_at: r.8.to_rfc3339(),
        })
        .collect();

    Ok(Json(rules))
}

/// GET /api/v1/rules/:id — Get a single rule
pub async fn get_rule(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<RuleResponse>, RuleError> {
    let row = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            Option<String>,
            String,
            String,
            f32,
            bool,
            chrono::DateTime<chrono::Utc>,
            chrono::DateTime<chrono::Utc>,
        ),
    >(
        r#"
        SELECT id, name, description, pattern, entity_label, confidence,
               is_active, created_at, updated_at
        FROM custom_rules
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(id)
    .bind(auth_user.user_id)
    .fetch_optional(state.db.pool())
    .await?
    .ok_or(RuleError::NotFound)?;

    Ok(Json(RuleResponse {
        id: row.0.to_string(),
        name: row.1,
        description: row.2,
        pattern: row.3,
        entity_label: row.4,
        confidence: row.5,
        is_active: row.6,
        created_at: row.7.to_rfc3339(),
        updated_at: row.8.to_rfc3339(),
    }))
}

/// PUT /api/v1/rules/:id — Update a rule
pub async fn update_rule(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateRuleRequest>,
) -> Result<Json<RuleResponse>, RuleError> {
    // Check rule exists and belongs to user
    let existing = sqlx::query_as::<_, (String, Option<String>, String, String, f32, bool)>(
        r#"
        SELECT name, description, pattern, entity_label, confidence, is_active
        FROM custom_rules
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(id)
    .bind(auth_user.user_id)
    .fetch_optional(state.db.pool())
    .await?
    .ok_or(RuleError::NotFound)?;

    let name = request.name.unwrap_or(existing.0);
    let description = request.description.or(existing.1);
    let pattern = request.pattern.unwrap_or(existing.2);
    let entity_label = request.entity_label.unwrap_or(existing.3);
    let confidence = request.confidence.unwrap_or(existing.4);
    let is_active = request.is_active.unwrap_or(existing.5);

    // Validate new pattern if changed
    CustomRuleDetector::validate_pattern(&pattern).map_err(RuleError::InvalidPattern)?;

    if !(0.0..=1.0).contains(&confidence) {
        return Err(RuleError::InvalidInput(
            "Confidence must be between 0.0 and 1.0".to_string(),
        ));
    }

    let row = sqlx::query_as::<_, (chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
        r#"
        UPDATE custom_rules
        SET name = $1, description = $2, pattern = $3, entity_label = $4,
            confidence = $5, is_active = $6, updated_at = NOW()
        WHERE id = $7 AND user_id = $8
        RETURNING created_at, updated_at
        "#,
    )
    .bind(&name)
    .bind(&description)
    .bind(&pattern)
    .bind(&entity_label)
    .bind(confidence)
    .bind(is_active)
    .bind(id)
    .bind(auth_user.user_id)
    .fetch_one(state.db.pool())
    .await?;

    Ok(Json(RuleResponse {
        id: id.to_string(),
        name,
        description,
        pattern,
        entity_label,
        confidence,
        is_active,
        created_at: row.0.to_rfc3339(),
        updated_at: row.1.to_rfc3339(),
    }))
}

/// DELETE /api/v1/rules/:id — Delete a rule
pub async fn delete_rule(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, RuleError> {
    let result = sqlx::query("DELETE FROM custom_rules WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(auth_user.user_id)
        .execute(state.db.pool())
        .await?;

    if result.rows_affected() == 0 {
        return Err(RuleError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/rules/:id/test — Test a rule against sample text
pub async fn test_rule(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<TestRuleRequest>,
) -> Result<Json<RuleTestResponse>, RuleError> {
    if request.text.is_empty() {
        return Err(RuleError::InvalidInput("Text cannot be empty".to_string()));
    }

    let row = sqlx::query_as::<_, (String, String, f32)>(
        r#"
        SELECT pattern, entity_label, confidence
        FROM custom_rules
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(id)
    .bind(auth_user.user_id)
    .fetch_optional(state.db.pool())
    .await?
    .ok_or(RuleError::NotFound)?;

    let rule = CustomRule {
        id,
        name: String::new(),
        pattern: row.0,
        entity_label: row.1.clone(),
        confidence: row.2,
    };

    let detector = CustomRuleDetector::new();
    let start = std::time::Instant::now();
    let entities = detector.detect(&request.text, &[rule]);
    let processing_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    let matches: Vec<RuleTestMatch> = entities
        .into_iter()
        .map(|e| RuleTestMatch {
            value: e.value,
            start: e.start,
            end: e.end,
            entity_label: e.custom_label.unwrap_or_default(),
            confidence: e.confidence.unwrap_or(0.0),
        })
        .collect();

    Ok(Json(RuleTestResponse {
        matches,
        processing_time_ms,
    }))
}

#[cfg(test)]
#[path = "rules_test.rs"]
mod tests;
