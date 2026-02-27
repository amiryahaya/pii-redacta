//! Authentication middleware for Axum
//!
//! Provides layers that can be applied to routes for API key
//! authentication and rate limiting.

use super::{extract_api_key, AuthError, AuthState, AuthenticatedUser};
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

/// Middleware function that authenticates and rate limits requests
///
/// This middleware:
/// 1. Extracts the API key from the Authorization header
/// 2. Validates the API key
/// 3. Checks rate limits
/// 4. Fetches user tier information
/// 5. Adds user info to request extensions for handlers
pub async fn api_auth_middleware(
    State(state): State<Arc<AuthState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    // Extract API key from headers
    let api_key = extract_api_key(request.headers())?;

    // Validate API key
    let validated = state
        .api_key_manager
        .validate_key(&api_key)
        .await
        .map_err(AuthError::ApiKey)?;

    // Check rate limit for this API key
    let user = get_user_info(&state, validated.user_id).await?;

    match state
        .rate_limiter
        .check_key_limit(
            validated.api_key.id,
            user.tier_features.rate_limit_per_minute,
        )
        .await
    {
        Ok(super::rate_limit::RateLimitResult::Allowed) => {}
        Ok(super::rate_limit::RateLimitResult::RetryAfter(_)) => {
            return Err(AuthError::RateLimitExceeded);
        }
        Err(e) => {
            tracing::warn!("Rate limiter error: {}", e);
            // Continue on rate limiter error (fail open)
        }
    }

    // Store authenticated user in request extensions
    request.extensions_mut().insert(user);

    Ok(next.run(request).await)
}

/// Lighter middleware that only validates API key without tier info
pub async fn simple_auth_middleware(
    State(state): State<Arc<AuthState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let api_key = extract_api_key(request.headers())?;

    let validated = state
        .api_key_manager
        .validate_key(&api_key)
        .await
        .map_err(AuthError::ApiKey)?;

    // Store minimal auth info
    request.extensions_mut().insert(SimpleAuth {
        user_id: validated.user_id,
        api_key_id: validated.api_key.id,
        environment: validated.environment,
    });

    Ok(next.run(request).await)
}

/// Middleware that applies rate limiting to unauthenticated requests (by IP)
pub async fn ip_rate_limit_middleware(
    State(state): State<Arc<AuthState>>,
    request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    // Extract client IP
    let ip = request
        .headers()
        .get("X-Forwarded-For")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            request
                .extensions()
                .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
                .map(|addr| addr.ip().to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());

    // Apply IP rate limit (10 requests per hour for unauthenticated)
    match state.rate_limiter.check_ip_limit(&ip, 10).await {
        Ok(super::rate_limit::RateLimitResult::Allowed) => {}
        Ok(super::rate_limit::RateLimitResult::RetryAfter(_)) => {
            return Err(AuthError::RateLimitExceeded);
        }
        Err(e) => {
            tracing::warn!("IP rate limiter error: {}", e);
        }
    }

    Ok(next.run(request).await)
}

/// Get user information including tier
async fn get_user_info(
    state: &AuthState,
    user_id: uuid::Uuid,
) -> Result<AuthenticatedUser, AuthError> {
    // Get user's subscription tier
    let subscription = sqlx::query_as::<_, (uuid::Uuid,)>(
        r#"
        SELECT tier_id FROM subscriptions 
        WHERE user_id = $1 AND status IN ('trial', 'active', 'past_due')
        "#,
    )
    .bind(user_id)
    .fetch_optional(state.api_key_manager.pool())
    .await
    .map_err(|e| {
        AuthError::ApiKey(pii_redacta_core::db::api_key_manager::ApiKeyError::Database(e))
    })?;

    let validated = state
        .api_key_manager
        .list_user_keys(user_id)
        .await
        .map_err(AuthError::ApiKey)?;

    let api_key_id = validated
        .first()
        .map(|k| k.id)
        .unwrap_or_else(uuid::Uuid::new_v4);

    let tier = match subscription {
        Some((tier_id,)) => state
            .tier_manager
            .get_by_id(tier_id)
            .await
            .map_err(AuthError::TierManager)?,
        None => {
            // Default to trial tier
            state
                .tier_manager
                .get_by_name("trial")
                .await
                .map_err(AuthError::TierManager)?
        }
    };

    Ok(AuthenticatedUser {
        user_id,
        api_key_id,
        environment: pii_redacta_core::db::api_key_manager::ApiKeyEnvironment::Live,
        tier_name: tier.name,
        tier_limits: tier.limits.0,
        tier_features: tier.features.0,
    })
}

/// Simple authentication info (without tier details)
#[derive(Debug, Clone)]
pub struct SimpleAuth {
    /// User ID
    pub user_id: uuid::Uuid,
    /// API key ID
    pub api_key_id: uuid::Uuid,
    /// Environment
    pub environment: pii_redacta_core::db::api_key_manager::ApiKeyEnvironment,
}

/// Extension trait to extract authentication info from requests
pub trait RequestAuthExt {
    /// Get the authenticated user (full info)
    fn authenticated_user(&self) -> Option<&AuthenticatedUser>;

    /// Get simple auth info
    fn simple_auth(&self) -> Option<&SimpleAuth>;
}

impl RequestAuthExt for axum::extract::Request {
    fn authenticated_user(&self) -> Option<&AuthenticatedUser> {
        self.extensions().get::<AuthenticatedUser>()
    }

    fn simple_auth(&self) -> Option<&SimpleAuth> {
        self.extensions().get::<SimpleAuth>()
    }
}

use crate::extractors::AuthUser;
use crate::jwt::{validate_token, JwtConfig};

/// JWT authentication middleware for portal routes
///
/// This middleware validates JWT tokens for browser-based authentication.
/// It's separate from API key middleware which is for programmatic access.
pub async fn jwt_auth_middleware(
    jwt_config: JwtConfig,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(header) => {
            crate::jwt::extract_token_from_header(header).ok_or(AuthError::MissingApiKey)?
        }
        None => return Err(AuthError::MissingApiKey),
    };

    // Validate token
    let claims = validate_token(token, &jwt_config).map_err(|_| AuthError::InvalidApiKeyFormat)?;

    // Parse user ID
    let user_id = claims
        .sub
        .parse()
        .map_err(|_| AuthError::InvalidApiKeyFormat)?;

    // Store user info in request extensions
    request.extensions_mut().insert(AuthUser {
        user_id,
        email: claims.email,
        is_admin: claims.is_admin,
    });

    Ok(next.run(request).await)
}

/// Re-export extractor functions
pub use super::extractors::{extract_api_auth, extract_simple_auth, try_extract_auth};
