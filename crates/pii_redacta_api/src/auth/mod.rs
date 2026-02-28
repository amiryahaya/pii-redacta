//! Authentication middleware for PII Redacta API
//!
//! Provides API key authentication and rate limiting for API endpoints.
//!
//! # Usage
//!
//! ```rust,ignore
//! use axum::{Router, routing::post};
//! use pii_redacta_api::auth::{api_auth_middleware, AuthState};
//!
//! let auth_state = AuthState::new(db, redis, server_secret).await?;
//!
//! let app = Router::new()
//!     .route("/api/detect", post(detect_handler))
//!     .layer(axum::middleware::from_fn_with_state(
//!         auth_state.clone(),
//!         api_auth_middleware
//!     ));
//! ```

pub mod errors;
pub mod extractors;
pub mod middleware;
pub mod rate_limit;

// Re-export commonly used types
pub use extractors::{
    extract_api_auth, extract_api_key, extract_simple_auth, try_extract_auth, SimpleAuth,
};
pub use middleware::{
    api_auth_middleware, ip_rate_limit_middleware, jwt_auth_middleware, simple_auth_middleware,
    RequestAuthExt,
};

use pii_redacta_core::db::api_key_manager::ApiKeyManager;
use pii_redacta_core::db::tier_manager::TierManager;
use pii_redacta_core::db::Database;
use rate_limit::RateLimiter;
use redis::Client as RedisClient;
use std::sync::Arc;

/// Authentication state shared across handlers
#[derive(Clone)]
pub struct AuthState {
    /// API key manager for validation
    pub api_key_manager: ApiKeyManager,
    /// Tier manager for limit checking
    pub tier_manager: TierManager,
    /// Rate limiter
    pub rate_limiter: Arc<RateLimiter>,
}

impl AuthState {
    /// Create a new authentication state
    pub async fn new(
        db: Arc<Database>,
        redis_url: &str,
        server_secret_b64: &str,
    ) -> Result<Self, AuthError> {
        let api_key_manager = ApiKeyManager::new(db.clone(), server_secret_b64)?;
        let tier_manager = TierManager::with_redis(db.clone(), redis_url)?;

        let redis_client = RedisClient::open(redis_url)?;
        let rate_limiter = Arc::new(RateLimiter::new(redis_client));

        Ok(Self {
            api_key_manager,
            tier_manager,
            rate_limiter,
        })
    }

    /// Create without Redis (for testing)
    pub fn new_without_redis(
        db: Arc<Database>,
        server_secret_b64: &str,
    ) -> Result<Self, AuthError> {
        let api_key_manager = ApiKeyManager::new(db.clone(), server_secret_b64)?;
        let tier_manager = TierManager::new(db.clone());

        // Create a dummy rate limiter (will fail if actually used)
        let redis_client = RedisClient::open("redis://localhost:6379")?;
        let rate_limiter = Arc::new(RateLimiter::new(redis_client));

        Ok(Self {
            api_key_manager,
            tier_manager,
            rate_limiter,
        })
    }
}

/// Authentication error types
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("API key error: {0}")]
    ApiKey(#[from] pii_redacta_core::db::api_key_manager::ApiKeyError),

    #[error("Tier manager error: {0}")]
    TierManager(#[from] pii_redacta_core::db::tier_manager::TierManagerError),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Missing API key")]
    MissingApiKey,

    #[error("Invalid API key format")]
    InvalidApiKeyFormat,

    #[error("Missing authentication token")]
    MissingToken,

    #[error("Authentication token has expired")]
    TokenExpired,

    #[error("Invalid authentication token")]
    InvalidToken,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Monthly file limit exceeded")]
    MonthlyLimitExceeded,

    #[error("File size limit exceeded")]
    FileSizeExceeded,
}

/// Authenticated user information extracted from API key
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    /// User ID
    pub user_id: uuid::Uuid,
    /// API key ID
    pub api_key_id: uuid::Uuid,
    /// Environment (live/test)
    pub environment: pii_redacta_core::db::api_key_manager::ApiKeyEnvironment,
    /// User's tier name
    pub tier_name: String,
    /// User's tier limits
    pub tier_limits: pii_redacta_core::db::models::TierLimits,
    /// User's tier features
    pub tier_features: pii_redacta_core::db::models::TierFeatures,
}

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AuthError::MissingApiKey => (
                StatusCode::UNAUTHORIZED,
                "Missing API key. Include 'Authorization: Bearer <key>' header.",
            ),
            AuthError::InvalidApiKeyFormat => (StatusCode::UNAUTHORIZED, "Invalid API key format."),
            AuthError::MissingToken => (
                StatusCode::UNAUTHORIZED,
                "Missing authentication token. Include 'Authorization: Bearer <token>' header.",
            ),
            AuthError::TokenExpired => (
                StatusCode::UNAUTHORIZED,
                "Authentication token has expired. Please log in again.",
            ),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid authentication token."),
            AuthError::ApiKey(e) => match e {
                pii_redacta_core::db::api_key_manager::ApiKeyError::NotFound => {
                    (StatusCode::UNAUTHORIZED, "Invalid API key.")
                }
                pii_redacta_core::db::api_key_manager::ApiKeyError::Expired => {
                    (StatusCode::UNAUTHORIZED, "API key has expired.")
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "Authentication error."),
            },
            AuthError::RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "Rate limit exceeded. Please try again later.",
            ),
            AuthError::MonthlyLimitExceeded => (
                StatusCode::PAYMENT_REQUIRED,
                "Monthly file limit exceeded. Please upgrade your plan.",
            ),
            AuthError::FileSizeExceeded => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "File size exceeds your plan limit.",
            ),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error."),
        };

        let body = serde_json::json!({
            "error": {
                "code": status.as_u16(),
                "message": error_message,
            }
        });

        (status, axum::Json(body)).into_response()
    }
}
