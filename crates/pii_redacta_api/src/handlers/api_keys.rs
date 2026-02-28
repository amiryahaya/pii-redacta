//! API Key management handlers for Portal
//!
//! Provides endpoints for users to manage their API keys through the web portal.

use crate::extractors::AuthUser;
use crate::AppState;
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use pii_redacta_core::db::api_key_manager::{ApiKeyEnvironment, ApiKeyManager};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Maximum allowed expiration period in days
const MAX_EXPIRES_DAYS: i32 = 365;
/// Minimum allowed expiration period in days
const MIN_EXPIRES_DAYS: i32 = 1;
/// Maximum API key name length
const MAX_KEY_NAME_LENGTH: usize = 100;
/// Maximum revocation reason length
const MAX_REVOKE_REASON_LENGTH: usize = 500;
/// Default page size for pagination
const DEFAULT_PAGE_LIMIT: i64 = 20;
/// Maximum page size for pagination
const MAX_PAGE_LIMIT: i64 = 100;

/// Pagination query parameters
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// API Key response for portal (without sensitive data)
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
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

/// Paginated API keys response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedApiKeysResponse {
    pub data: Vec<ApiKeyResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Generated API key response (includes full key - shown once)
/// Note: Debug intentionally not derived to prevent full_key from appearing in logs (S9-R3-08)
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
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
    #[error("Validation error: {0}")]
    Validation(String),
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
            ApiKeyHandlerError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
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

/// List all API keys for the authenticated user with pagination
pub async fn list_api_keys(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedApiKeysResponse>, ApiKeyHandlerError> {
    let limit = params
        .limit
        .unwrap_or(DEFAULT_PAGE_LIMIT)
        .clamp(1, MAX_PAGE_LIMIT);
    let offset = params.offset.unwrap_or(0).max(0);

    let api_key_manager = ApiKeyManager::new(state.db.clone(), &state.api_key_secret)?;

    let (keys, total) = api_key_manager
        .list_user_keys_paginated(auth_user.user_id, limit, offset)
        .await?;

    let data: Vec<ApiKeyResponse> = keys
        .into_iter()
        .map(|key| ApiKeyResponse {
            id: key.id.to_string(),
            name: key.name,
            key_prefix: key.key_prefix,
            environment: key.environment.clone(),
            last_used_at: key.last_used_at.map(|d| d.to_rfc3339()),
            expires_at: key.expires_at.map(|d| d.to_rfc3339()),
            is_active: key.is_active,
            created_at: key.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(PaginatedApiKeysResponse {
        data,
        total,
        limit,
        offset,
    }))
}

/// Create a new API key
pub async fn create_api_key(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateApiKeyRequest>,
) -> Result<Json<CreateApiKeyResponse>, ApiKeyHandlerError> {
    // Validate name (S9-R2-11)
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return Err(ApiKeyHandlerError::Validation(
            "API key name cannot be empty".to_string(),
        ));
    }
    if name.len() > MAX_KEY_NAME_LENGTH {
        return Err(ApiKeyHandlerError::Validation(format!(
            "API key name must be less than {} characters",
            MAX_KEY_NAME_LENGTH
        )));
    }

    // Validate environment
    let environment = match req.environment.as_str() {
        "live" => ApiKeyEnvironment::Live,
        "test" => ApiKeyEnvironment::Test,
        _ => return Err(ApiKeyHandlerError::InvalidEnvironment),
    };

    // Validate expires_days bounds (S9-28)
    if let Some(days) = req.expires_days {
        if !(MIN_EXPIRES_DAYS..=MAX_EXPIRES_DAYS).contains(&days) {
            return Err(ApiKeyHandlerError::Validation(format!(
                "expires_days must be between {} and {}",
                MIN_EXPIRES_DAYS, MAX_EXPIRES_DAYS
            )));
        }
    }

    let api_key_manager = ApiKeyManager::new(state.db.clone(), &state.api_key_secret)?;

    // Calculate expiration
    let expires_at = req
        .expires_days
        .map(|days| Utc::now() + chrono::Duration::days(days as i64));

    // Check user's tier limit for API keys
    // S9-R4-11: Minor TOCTOU race between count and creation is acceptable —
    // worst case is one extra key beyond the limit under concurrent requests.
    let subscription = sqlx::query_as::<_, (Uuid,)>(
        r#"
        SELECT s.tier_id FROM subscriptions s
        JOIN tiers t ON s.tier_id = t.id
        WHERE s.user_id = $1 AND s.status IN ('trial', 'active', 'past_due')
        ORDER BY s.created_at DESC
        LIMIT 1
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

    // Generate API key — generate_key returns id and created_at from RETURNING clause
    let generated = api_key_manager
        .generate_key(auth_user.user_id, &name, environment, expires_at)
        .await?;

    Ok(Json(CreateApiKeyResponse {
        id: generated.id.to_string(),
        name,
        full_key: generated.full_key,
        key_prefix: generated.prefix,
        environment: match generated.environment {
            ApiKeyEnvironment::Live => "live".to_string(),
            ApiKeyEnvironment::Test => "test".to_string(),
        },
        expires_at: generated.expires_at.map(|d| d.to_rfc3339()),
        created_at: generated.created_at.to_rfc3339(),
    }))
}

/// Delete (revoke) an API key without a request body (S9-25: propagate errors properly)
pub async fn delete_api_key(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(key_id): Path<Uuid>,
) -> Result<StatusCode, ApiKeyHandlerError> {
    let api_key_manager = ApiKeyManager::new(state.db.clone(), &state.api_key_secret)?;

    // Verify the key exists and belongs to the user — propagate DB errors (S9-25)
    let key = api_key_manager
        .get_key(key_id, auth_user.user_id)
        .await
        .map_err(|e| match e {
            pii_redacta_core::db::api_key_manager::ApiKeyError::NotFound => {
                ApiKeyHandlerError::NotFound
            }
            other => ApiKeyHandlerError::ApiKeyManager(other),
        })?;

    api_key_manager
        .revoke_key(key.id, auth_user.user_id, None)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Revoke an API key (S9-25: propagate errors properly)
pub async fn revoke_api_key(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(key_id): Path<Uuid>,
    Json(req): Json<RevokeApiKeyRequest>,
) -> Result<StatusCode, ApiKeyHandlerError> {
    // Validate revocation reason length (S9-R2-23)
    if let Some(ref reason) = req.reason {
        if reason.len() > MAX_REVOKE_REASON_LENGTH {
            return Err(ApiKeyHandlerError::Validation(format!(
                "Revocation reason must be less than {} characters",
                MAX_REVOKE_REASON_LENGTH
            )));
        }
    }

    let api_key_manager = ApiKeyManager::new(state.db.clone(), &state.api_key_secret)?;

    // Verify the key belongs to the user — propagate DB errors (S9-25)
    let key = api_key_manager
        .get_key(key_id, auth_user.user_id)
        .await
        .map_err(|e| match e {
            pii_redacta_core::db::api_key_manager::ApiKeyError::NotFound => {
                ApiKeyHandlerError::NotFound
            }
            other => ApiKeyHandlerError::ApiKeyManager(other),
        })?;

    api_key_manager
        .revoke_key(key.id, auth_user.user_id, req.reason.as_deref())
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expires_days_bounds() {
        // Verify constants are sensible at compile-time
        const _: () = assert!(MIN_EXPIRES_DAYS > 0);
        const _: () = assert!(MAX_EXPIRES_DAYS <= 365);
    }

    #[test]
    fn test_key_name_length_limit() {
        const _: () = assert!(MAX_KEY_NAME_LENGTH > 0);
        const _: () = assert!(MAX_KEY_NAME_LENGTH <= 255);
    }

    #[test]
    fn test_revoke_reason_length_limit() {
        const _: () = assert!(MAX_REVOKE_REASON_LENGTH > 0);
        const _: () = assert!(MAX_REVOKE_REASON_LENGTH <= 1000);
    }

    #[test]
    fn test_pagination_defaults() {
        const _: () = assert!(DEFAULT_PAGE_LIMIT > 0);
        const _: () = assert!(MAX_PAGE_LIMIT >= DEFAULT_PAGE_LIMIT);
    }

    #[test]
    fn test_environment_parsing() {
        // Valid environments
        assert!(matches!("live".parse::<String>().as_deref(), Ok("live")));
        assert!(matches!("test".parse::<String>().as_deref(), Ok("test")));

        // Simulate the handler's match logic
        let valid_envs = |s: &str| -> bool { matches!(s, "live" | "test") };
        assert!(valid_envs("live"));
        assert!(valid_envs("test"));
        assert!(!valid_envs("staging"));
        assert!(!valid_envs("production"));
        assert!(!valid_envs(""));
    }

    #[test]
    fn test_error_response_status_codes() {
        use axum::response::IntoResponse;

        let cases: Vec<(ApiKeyHandlerError, axum::http::StatusCode)> = vec![
            (
                ApiKeyHandlerError::InvalidEnvironment,
                axum::http::StatusCode::BAD_REQUEST,
            ),
            (
                ApiKeyHandlerError::NotFound,
                axum::http::StatusCode::NOT_FOUND,
            ),
            (
                ApiKeyHandlerError::MaxKeysReached,
                axum::http::StatusCode::FORBIDDEN,
            ),
            (
                ApiKeyHandlerError::Validation("test".to_string()),
                axum::http::StatusCode::BAD_REQUEST,
            ),
            (
                ApiKeyHandlerError::Database(sqlx::Error::RowNotFound),
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            ),
        ];

        for (error, expected_status) in cases {
            let response = error.into_response();
            assert_eq!(
                response.status(),
                expected_status,
                "Unexpected status for error variant"
            );
        }
    }
}
