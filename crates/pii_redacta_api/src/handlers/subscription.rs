//! Subscription and tier handlers for Portal
//!
//! Provides endpoints for users to view their subscription and available tiers.

use crate::extractors::AuthUser;
use crate::AppState;
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use serde_json::Value;

/// Error type for subscription handlers
#[derive(Debug, thiserror::Error)]
pub enum SubscriptionError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("No active subscription found")]
    NotFound,
}

impl IntoResponse for SubscriptionError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            SubscriptionError::NotFound => (
                StatusCode::NOT_FOUND,
                "No active subscription found. Please subscribe to a plan.",
            ),
            SubscriptionError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unexpected error occurred",
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

/// Subscription response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionResponse {
    pub id: String,
    pub status: String,
    pub tier: TierResponse,
    pub current_period_start: Option<String>,
    pub current_period_end: Option<String>,
    pub cancel_at_period_end: bool,
}

/// Tier response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TierResponse {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub limits: Value,
    pub features: Value,
    pub monthly_price_cents: Option<i32>,
    pub yearly_price_cents: Option<i32>,
}

/// Named row struct for subscription query (replaces fragile 13-element tuple)
#[derive(sqlx::FromRow)]
struct SubscriptionRow {
    sub_id: uuid::Uuid,
    status: String,
    tier_id: uuid::Uuid,
    tier_name: String,
    display_name: String,
    description: Option<String>,
    limits: Value,
    features: Value,
    monthly_price_cents: Option<i32>,
    yearly_price_cents: Option<i32>,
    current_period_start: Option<chrono::DateTime<chrono::Utc>>,
    current_period_end: Option<chrono::DateTime<chrono::Utc>>,
    cancel_at_period_end: bool,
}

/// Get current user's subscription
pub async fn get_subscription(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<SubscriptionResponse>, SubscriptionError> {
    let row = sqlx::query_as::<_, SubscriptionRow>(
        r#"
        SELECT
            s.id as sub_id,
            s.status::text as status,
            t.id as tier_id,
            t.name as tier_name,
            t.display_name,
            t.description,
            t.limits,
            t.features,
            t.monthly_price_cents,
            t.yearly_price_cents,
            s.current_period_start,
            s.current_period_end,
            s.cancel_at_period_end
        FROM subscriptions s
        JOIN tiers t ON s.tier_id = t.id
        WHERE s.user_id = $1
        AND s.status IN ('trial', 'active', 'past_due')
        ORDER BY s.created_at DESC
        LIMIT 1
        "#,
    )
    .bind(auth_user.user_id)
    .fetch_optional(state.db.pool())
    .await?;

    let row = row.ok_or(SubscriptionError::NotFound)?;

    Ok(Json(SubscriptionResponse {
        id: row.sub_id.to_string(),
        status: row.status,
        tier: TierResponse {
            id: row.tier_id.to_string(),
            name: row.tier_name,
            display_name: row.display_name,
            description: row.description,
            limits: row.limits,
            features: row.features,
            monthly_price_cents: row.monthly_price_cents,
            yearly_price_cents: row.yearly_price_cents,
        },
        current_period_start: row.current_period_start.map(|d| d.to_rfc3339()),
        current_period_end: row.current_period_end.map(|d| d.to_rfc3339()),
        cancel_at_period_end: row.cancel_at_period_end,
    }))
}

/// List all available tiers
pub async fn list_tiers(
    State(state): State<AppState>,
) -> Result<Json<Vec<TierResponse>>, SubscriptionError> {
    let rows = sqlx::query_as::<
        _,
        (
            uuid::Uuid,
            String,
            String,
            Option<String>,
            Value,
            Value,
            Option<i32>,
            Option<i32>,
        ),
    >(
        r#"
        SELECT
            id,
            name,
            display_name,
            description,
            limits,
            features,
            monthly_price_cents,
            yearly_price_cents
        FROM tiers
        WHERE is_active = true AND is_public = true
        ORDER BY sort_order ASC
        "#,
    )
    .fetch_all(state.db.pool())
    .await?;

    let tiers: Vec<TierResponse> = rows
        .into_iter()
        .map(|row| TierResponse {
            id: row.0.to_string(),
            name: row.1,
            display_name: row.2,
            description: row.3,
            limits: row.4,
            features: row.5,
            monthly_price_cents: row.6,
            yearly_price_cents: row.7,
        })
        .collect();

    Ok(Json(tiers))
}
