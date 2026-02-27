//! API Key management handlers for Portal
//!
//! Provides endpoints for users to manage their API keys through the web portal.

use crate::extractors::AuthUser;
use crate::AppState;
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use pii_redacta_core::db::api_key_manager::{ApiKeyEnvironment, ApiKeyManager};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// API Key response for portal (without sensitive data)
#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    pub id: String,
    pub name: String,
    pub key_prefix: String,
    pub environment: String,
    pub last_used_at: Option<String>,
    pub expires_at: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

/// Generated API key response (includes full key - shown once)
#[derive(Debug, Serialize)]
pub struct CreateApiKeyResponse {
    pub id: String,
    pub name: String,
    pub full_key: String,
    pub key_prefix: String,
    pub environment: String,
    pub expires_at: Option<String>,
    pub created_at: String,
}

/// Create API key request
#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub environment: String,       // "live" or "test"
    pub expires_days: Option<i32>, // Optional expiration in days
}

/// Revoke API key request
#[derive(Debug, Deserialize)]
pub struct RevokeApiKeyRequest {
    pub reason: Option<String>,
}

/// Error types for API key handlers
#[derive(Debug, thiserror::Error)]
pub enum ApiKeyHandlerError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("API key manager error: {0}")]
    ApiKeyManager(#[from] pii_redacta_core::db::api_key_manager::ApiKeyError),
    #[error("Invalid environment")]
    InvalidEnvironment,
    #[error("API key not found")]
    NotFound,
    #[error("Maximum number of API keys reached")]
    MaxKeysReached,
}

impl IntoResponse for ApiKeyHandlerError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            ApiKeyHandlerError::InvalidEnvironment => (
                StatusCode::BAD_REQUEST,
                "Environment must be 'live' or 'test'",
            ),
            ApiKeyHandlerError::NotFound => (StatusCode::NOT_FOUND, "API key not found"),
            ApiKeyHandlerError::MaxKeysReached => (
                StatusCode::FORBIDDEN,
                "Maximum number of API keys reached for your plan",
            ),
            _ => (
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

/// List all API keys for the authenticated user
pub async fn list_api_keys(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Vec<ApiKeyResponse>>, ApiKeyHandlerError> {
    // Create API key manager
    let api_key_manager = ApiKeyManager::new(
        state.db.clone(),
        state.jwt_config.secret(), // Reuse JWT secret for API key HMAC
    )?;

    let keys = api_key_manager.list_user_keys(auth_user.user_id).await?;

    let responses: Vec<ApiKeyResponse> = keys
        .into_iter()
        .map(|key| {
            // Extract environment from key prefix format: pii_{env}_{prefix}_{secret}
            // For now, derive from key_prefix or default to 'live'
            let environment = if key.name.to_lowercase().contains("test") {
                "test"
            } else {
                "live"
            }
            .to_string();

            ApiKeyResponse {
                id: key.id.to_string(),
                name: key.name,
                key_prefix: key.key_prefix,
                environment,
                last_used_at: key.last_used_at.map(|d| d.to_rfc3339()),
                expires_at: key.expires_at.map(|d| d.to_rfc3339()),
                is_active: key.is_active,
                created_at: key.created_at.to_rfc3339(),
            }
        })
        .collect();

    Ok(Json(responses))
}

/// Create a new API key
pub async fn create_api_key(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateApiKeyRequest>,
) -> Result<Json<CreateApiKeyResponse>, ApiKeyHandlerError> {
    // Validate environment
    let environment = match req.environment.as_str() {
        "live" => ApiKeyEnvironment::Live,
        "test" => ApiKeyEnvironment::Test,
        _ => return Err(ApiKeyHandlerError::InvalidEnvironment),
    };

    // Create API key manager
    let api_key_manager = ApiKeyManager::new(state.db.clone(), state.jwt_config.secret())?;

    // Check user's tier limit for API keys
    let subscription = sqlx::query_as::<_, (Uuid,)>(
        r#"
        SELECT s.tier_id FROM subscriptions s
        JOIN tiers t ON s.tier_id = t.id
        WHERE s.user_id = $1 AND s.status IN ('trial', 'active', 'past_due')
        "#,
    )
    .bind(auth_user.user_id)
    .fetch_optional(state.db.pool())
    .await?;

    if let Some((tier_id,)) = subscription {
        let tier =
            sqlx::query_as::<_, (serde_json::Value,)>("SELECT limits FROM tiers WHERE id = $1")
                .bind(tier_id)
                .fetch_one(state.db.pool())
                .await?;

        // Check max_api_keys limit
        if let Some(max_keys) = tier.0.get("max_api_keys").and_then(|v| v.as_i64()) {
            let current_count = api_key_manager.count_user_keys(auth_user.user_id).await?;
            if current_count >= max_keys {
                return Err(ApiKeyHandlerError::MaxKeysReached);
            }
        }
    }

    // Calculate expiration
    let expires_at = req
        .expires_days
        .map(|days| Utc::now() + chrono::Duration::days(days as i64));

    // Generate API key
    let generated = api_key_manager
        .generate_key(auth_user.user_id, &req.name, environment, expires_at)
        .await?;

    // Query the newly created key to get its actual ID
    let created_key = sqlx::query_as::<_, pii_redacta_core::db::models::ApiKey>(
        r#"
        SELECT 
            id, user_id, key_prefix, key_hash, name,
            last_used_at, expires_at, is_active, revoked_at,
            revoked_reason, created_at
        FROM api_keys 
        WHERE user_id = $1 AND key_prefix = $2
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(auth_user.user_id)
    .bind(&generated.prefix)
    .fetch_one(state.db.pool())
    .await
    .map_err(ApiKeyHandlerError::Database)?;

    Ok(Json(CreateApiKeyResponse {
        id: created_key.id.to_string(),
        name: req.name,
        full_key: generated.full_key,
        key_prefix: generated.prefix,
        environment: match generated.environment {
            ApiKeyEnvironment::Live => "live".to_string(),
            ApiKeyEnvironment::Test => "test".to_string(),
        },
        expires_at: generated.expires_at.map(|d| d.to_rfc3339()),
        created_at: created_key.created_at.to_rfc3339(),
    }))
}

/// Delete (revoke) an API key without a request body
pub async fn delete_api_key(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(key_id): Path<Uuid>,
) -> Result<StatusCode, ApiKeyHandlerError> {
    let api_key_manager = ApiKeyManager::new(state.db.clone(), state.jwt_config.secret())?;

    let key = api_key_manager.get_key(key_id, auth_user.user_id).await;

    match key {
        Ok(_) => {
            api_key_manager
                .revoke_key(key_id, auth_user.user_id, None)
                .await?;
            Ok(StatusCode::NO_CONTENT)
        }
        Err(_) => Err(ApiKeyHandlerError::NotFound),
    }
}

/// Revoke an API key
pub async fn revoke_api_key(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(key_id): Path<Uuid>,
    Json(req): Json<RevokeApiKeyRequest>,
) -> Result<StatusCode, ApiKeyHandlerError> {
    let api_key_manager = ApiKeyManager::new(state.db.clone(), state.jwt_config.secret())?;

    // Verify the key belongs to the user
    let key = api_key_manager.get_key(key_id, auth_user.user_id).await;

    match key {
        Ok(_) => {
            // Key exists and belongs to user, revoke it
            api_key_manager
                .revoke_key(key_id, auth_user.user_id, req.reason.as_deref())
                .await?;
            Ok(StatusCode::NO_CONTENT)
        }
        Err(_) => Err(ApiKeyHandlerError::NotFound),
    }
}
