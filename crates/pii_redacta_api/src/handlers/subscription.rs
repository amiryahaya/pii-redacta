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
}

impl IntoResponse for SubscriptionError {
    fn into_response(self) -> axum::response::Response {
        let body = serde_json::json!({
            "error": {
                "code": 500,
                "message": "An unexpected error occurred",
            }
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
    }
}

/// Subscription response
#[derive(Debug, Serialize)]
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

/// Get current user's subscription
pub async fn get_subscription(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<SubscriptionResponse>, SubscriptionError> {
    let row = sqlx::query_as::<
        _,
        (
            uuid::Uuid,
            String,
            uuid::Uuid,
            String,
            String,
            Option<String>,
            Value,
            Value,
            Option<i32>,
            Option<i32>,
            Option<chrono::DateTime<chrono::Utc>>,
            Option<chrono::DateTime<chrono::Utc>>,
            bool,
        ),
    >(
        r#"
        SELECT 
            s.id,
            s.status,
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
    .fetch_one(state.db.pool())
    .await?;

    Ok(Json(SubscriptionResponse {
        id: row.0.to_string(),
        status: row.1,
        tier: TierResponse {
            id: row.2.to_string(),
            name: row.3,
            display_name: row.4,
            description: row.5,
            limits: row.6,
            features: row.7,
            monthly_price_cents: row.8,
            yearly_price_cents: row.9,
        },
        current_period_start: row.10.map(|d| d.to_rfc3339()),
        current_period_end: row.11.map(|d| d.to_rfc3339()),
        cancel_at_period_end: row.12,
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
