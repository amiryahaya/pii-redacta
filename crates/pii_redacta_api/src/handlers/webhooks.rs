//! Webhooks handler
//!
//! Sprint 14: CRUD for webhook endpoints, delivery log, and test sending.

use crate::extractors::AuthUser;
use crate::AppState;
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Request / Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWebhookRequest {
    pub url: String,
    pub description: Option<String>,
    pub events: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookResponse {
    pub id: String,
    pub url: String,
    pub description: Option<String>,
    pub secret: String,
    pub events: Vec<String>,
    pub is_active: bool,
    pub failure_count: i32,
    pub last_triggered_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookDeliveryResponse {
    pub id: String,
    pub event_type: String,
    pub status: String,
    pub http_status: Option<i32>,
    pub attempts: i32,
    pub created_at: String,
    pub delivered_at: Option<String>,
}

// ============================================================================
// Error Type
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum WebhookError {
    #[error("Webhooks are not available on your current plan")]
    NotAvailable,
    #[error("Webhook endpoint limit reached for your plan")]
    LimitReached,
    #[error("Invalid webhook URL: {0}")]
    InvalidUrl(String),
    #[error("Webhook not found")]
    NotFound,
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for WebhookError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            WebhookError::NotAvailable => (StatusCode::FORBIDDEN, self.to_string()),
            WebhookError::LimitReached => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
            WebhookError::InvalidUrl(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            WebhookError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            WebhookError::Database(ref e) => {
                tracing::error!("Webhook database error: {e}");
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
// SSRF Protection
// ============================================================================

/// Validate webhook URL: must be HTTPS, reject private IPs and localhost.
fn validate_webhook_url(url_str: &str) -> Result<(), String> {
    let parsed = url::Url::parse(url_str).map_err(|e| format!("Invalid URL: {e}"))?;

    if parsed.scheme() != "https" {
        return Err("Webhook URL must use HTTPS".to_string());
    }

    let host = parsed.host_str().ok_or("URL must have a host")?;

    // Reject localhost
    if host == "localhost" || host == "127.0.0.1" || host == "::1" || host == "[::1]" {
        return Err("Private/localhost URLs are not allowed".to_string());
    }

    // Check for private IP ranges
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        if is_private_ip(&ip) {
            return Err("Private IP addresses are not allowed".to_string());
        }
    }

    Ok(())
}

fn is_private_ip(ip: &std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => {
            v4.is_loopback() || v4.is_private() || v4.is_link_local() || v4.octets()[0] == 0
        }
        std::net::IpAddr::V6(v6) => v6.is_loopback(),
    }
}

/// Generate a random webhook secret (hex-encoded 32 bytes)
fn generate_webhook_secret() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    hex::encode(bytes)
}

// ============================================================================
// Tier Gating Helper
// ============================================================================

async fn check_webhook_access(
    state: &AppState,
    user_id: Uuid,
) -> Result<pii_redacta_core::db::models::TierLimits, WebhookError> {
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

    let (limits_json, features_json) = row.ok_or(WebhookError::NotAvailable)?;

    let features: pii_redacta_core::db::models::TierFeatures =
        serde_json::from_value(features_json).unwrap_or_default();
    if !features.webhooks {
        return Err(WebhookError::NotAvailable);
    }

    let limits: pii_redacta_core::db::models::TierLimits =
        serde_json::from_value(limits_json).unwrap_or_default();

    Ok(limits)
}

// ============================================================================
// Handlers
// ============================================================================

/// POST /api/v1/webhooks — Create a webhook endpoint
pub async fn create_webhook(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<CreateWebhookRequest>,
) -> Result<(StatusCode, Json<WebhookResponse>), WebhookError> {
    let limits = check_webhook_access(&state, auth_user.user_id).await?;

    // Validate URL (SSRF protection)
    validate_webhook_url(&request.url).map_err(WebhookError::InvalidUrl)?;

    // Check endpoint count limit
    if let Some(max_endpoints) = limits.max_webhook_endpoints {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM webhook_endpoints WHERE user_id = $1")
                .bind(auth_user.user_id)
                .fetch_one(state.db.pool())
                .await?;

        if count >= max_endpoints as i64 {
            return Err(WebhookError::LimitReached);
        }
    }

    let secret = generate_webhook_secret();

    let row = sqlx::query_as::<_, (Uuid, chrono::DateTime<chrono::Utc>)>(
        r#"
        INSERT INTO webhook_endpoints (user_id, url, description, secret, events)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, created_at
        "#,
    )
    .bind(auth_user.user_id)
    .bind(&request.url)
    .bind(&request.description)
    .bind(&secret)
    .bind(&request.events)
    .fetch_one(state.db.pool())
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(WebhookResponse {
            id: row.0.to_string(),
            url: request.url,
            description: request.description,
            secret,
            events: request.events,
            is_active: true,
            failure_count: 0,
            last_triggered_at: None,
            created_at: row.1.to_rfc3339(),
        }),
    ))
}

/// GET /api/v1/webhooks — List user's webhooks
pub async fn list_webhooks(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Vec<WebhookResponse>>, WebhookError> {
    check_webhook_access(&state, auth_user.user_id).await?;

    let rows = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            Option<String>,
            String,
            Vec<String>,
            bool,
            i32,
            Option<chrono::DateTime<chrono::Utc>>,
            chrono::DateTime<chrono::Utc>,
        ),
    >(
        r#"
        SELECT id, url, description, secret, events, is_active,
               failure_count, last_triggered_at, created_at
        FROM webhook_endpoints
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(auth_user.user_id)
    .fetch_all(state.db.pool())
    .await?;

    let webhooks: Vec<WebhookResponse> = rows
        .into_iter()
        .map(|r| WebhookResponse {
            id: r.0.to_string(),
            url: r.1,
            description: r.2,
            secret: mask_secret(&r.3),
            events: r.4,
            is_active: r.5,
            failure_count: r.6,
            last_triggered_at: r.7.map(|t| t.to_rfc3339()),
            created_at: r.8.to_rfc3339(),
        })
        .collect();

    Ok(Json(webhooks))
}

/// Mask a webhook secret for display (show first 8 chars)
fn mask_secret(secret: &str) -> String {
    if secret.len() > 8 {
        format!("{}...", &secret[..8])
    } else {
        "***".to_string()
    }
}

/// GET /api/v1/webhooks/:id — Get a single webhook
pub async fn get_webhook(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<WebhookResponse>, WebhookError> {
    let row = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            Option<String>,
            String,
            Vec<String>,
            bool,
            i32,
            Option<chrono::DateTime<chrono::Utc>>,
            chrono::DateTime<chrono::Utc>,
        ),
    >(
        r#"
        SELECT id, url, description, secret, events, is_active,
               failure_count, last_triggered_at, created_at
        FROM webhook_endpoints
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(id)
    .bind(auth_user.user_id)
    .fetch_optional(state.db.pool())
    .await?
    .ok_or(WebhookError::NotFound)?;

    Ok(Json(WebhookResponse {
        id: row.0.to_string(),
        url: row.1,
        description: row.2,
        secret: mask_secret(&row.3),
        events: row.4,
        is_active: row.5,
        failure_count: row.6,
        last_triggered_at: row.7.map(|t| t.to_rfc3339()),
        created_at: row.8.to_rfc3339(),
    }))
}

/// DELETE /api/v1/webhooks/:id — Delete a webhook endpoint and its deliveries
pub async fn delete_webhook(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, WebhookError> {
    let result = sqlx::query("DELETE FROM webhook_endpoints WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(auth_user.user_id)
        .execute(state.db.pool())
        .await?;

    if result.rows_affected() == 0 {
        return Err(WebhookError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/webhooks/:id/test — Send a test webhook event
pub async fn test_webhook(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, WebhookError> {
    // Verify ownership
    let row = sqlx::query_as::<_, (String, String, bool)>(
        "SELECT url, secret, is_active FROM webhook_endpoints WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(auth_user.user_id)
    .fetch_optional(state.db.pool())
    .await?
    .ok_or(WebhookError::NotFound)?;

    let (url, secret, _is_active) = row;

    // Create test delivery record
    let test_payload = serde_json::json!({
        "event": "test",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "data": {
            "message": "This is a test webhook delivery"
        }
    });

    let delivery_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO webhook_deliveries (endpoint_id, event_type, payload, status, max_attempts)
        VALUES ($1, 'test', $2, 'pending', 1)
        RETURNING id
        "#,
    )
    .bind(id)
    .bind(&test_payload)
    .fetch_one(state.db.pool())
    .await?;

    // Attempt delivery
    let result =
        crate::webhook_delivery::deliver_webhook(&url, &secret, delivery_id, "test", &test_payload)
            .await;

    match result {
        Ok(status_code) => {
            let _ = sqlx::query(
                r#"
                UPDATE webhook_deliveries
                SET status = 'delivered', http_status = $1, attempts = 1, delivered_at = NOW()
                WHERE id = $2
                "#,
            )
            .bind(status_code as i32)
            .bind(delivery_id)
            .execute(state.db.pool())
            .await;

            Ok(Json(serde_json::json!({
                "success": true,
                "deliveryId": delivery_id.to_string(),
                "httpStatus": status_code,
            })))
        }
        Err(e) => {
            let err_msg: String = e.to_string().chars().take(500).collect();
            let _ = sqlx::query(
                r#"
                UPDATE webhook_deliveries
                SET status = 'failed', response_body = $1, attempts = 1
                WHERE id = $2
                "#,
            )
            .bind(&err_msg)
            .bind(delivery_id)
            .execute(state.db.pool())
            .await;

            Ok(Json(serde_json::json!({
                "success": false,
                "deliveryId": delivery_id.to_string(),
                "error": err_msg,
            })))
        }
    }
}

/// GET /api/v1/webhooks/:id/deliveries — List recent deliveries
pub async fn list_deliveries(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<WebhookDeliveryResponse>>, WebhookError> {
    // Verify ownership
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM webhook_endpoints WHERE id = $1 AND user_id = $2)",
    )
    .bind(id)
    .bind(auth_user.user_id)
    .fetch_one(state.db.pool())
    .await?;

    if !exists {
        return Err(WebhookError::NotFound);
    }

    let rows = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            String,
            Option<i32>,
            i32,
            chrono::DateTime<chrono::Utc>,
            Option<chrono::DateTime<chrono::Utc>>,
        ),
    >(
        r#"
        SELECT id, event_type, status, http_status, attempts,
               created_at, delivered_at
        FROM webhook_deliveries
        WHERE endpoint_id = $1
        ORDER BY created_at DESC
        LIMIT 20
        "#,
    )
    .bind(id)
    .fetch_all(state.db.pool())
    .await?;

    let deliveries: Vec<WebhookDeliveryResponse> = rows
        .into_iter()
        .map(|r| WebhookDeliveryResponse {
            id: r.0.to_string(),
            event_type: r.1,
            status: r.2,
            http_status: r.3,
            attempts: r.4,
            created_at: r.5.to_rfc3339(),
            delivered_at: r.6.map(|t| t.to_rfc3339()),
        })
        .collect();

    Ok(Json(deliveries))
}

#[cfg(test)]
#[path = "webhooks_test.rs"]
mod tests;
