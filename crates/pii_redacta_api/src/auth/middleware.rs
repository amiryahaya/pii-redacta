//! Authentication middleware for Axum
//!
//! Provides layers that can be applied to routes for API key
//! authentication and rate limiting.

use super::{extract_api_key, extractors::SimpleAuth, AuthError, AuthState, AuthenticatedUser};
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

    // Get user info using the validated key's ID and environment (S9-R3-04)
    let user = get_user_info(
        &state,
        validated.user_id,
        validated.api_key.id,
        validated.environment,
    )
    .await?;

    // Check rate limit for this API key
    // Fail closed: if Redis is down, reject the request (S9-R3-01)
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
            tracing::error!("API key rate limiter error (failing closed): {}", e);
            return Err(AuthError::RateLimitExceeded);
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
    // Extract client IP using ConnectInfo (S12-3: XFF only trusted from known proxies)
    let ip = request
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Apply IP rate limit (10 requests per hour for unauthenticated)
    // Fail closed: if Redis is down, reject the request (S9-R2-03)
    match state.rate_limiter.check_ip_limit(&ip, 10).await {
        Ok(super::rate_limit::RateLimitResult::Allowed) => {}
        Ok(super::rate_limit::RateLimitResult::RetryAfter(_)) => {
            return Err(AuthError::RateLimitExceeded);
        }
        Err(e) => {
            tracing::error!("IP rate limiter error (failing closed): {}", e);
            return Err(AuthError::RateLimitExceeded);
        }
    }

    Ok(next.run(request).await)
}

/// Get user information including tier (S9-R3-04: accepts validated key ID/env from caller)
async fn get_user_info(
    state: &AuthState,
    user_id: uuid::Uuid,
    api_key_id: uuid::Uuid,
    environment: pii_redacta_core::db::api_key_manager::ApiKeyEnvironment,
) -> Result<AuthenticatedUser, AuthError> {
    // Get user's subscription tier
    let subscription = sqlx::query_as::<_, (uuid::Uuid,)>(
        r#"
        SELECT tier_id FROM subscriptions
        WHERE user_id = $1 AND status IN ('trial', 'active', 'past_due')
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(state.api_key_manager.pool())
    .await
    .map_err(|e| {
        AuthError::ApiKey(pii_redacta_core::db::api_key_manager::ApiKeyError::Database(e))
    })?;

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
        environment,
        tier_name: tier.name,
        tier_limits: tier.limits.0,
        tier_features: tier.features.0,
    })
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
use crate::AppState;

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
            crate::jwt::extract_token_from_header(header).ok_or(AuthError::MissingToken)?
        }
        None => return Err(AuthError::MissingToken),
    };

    // Validate token — map specific JWT errors to appropriate AuthError variants (S9-31)
    let claims = validate_token(token, &jwt_config).map_err(|e| match e {
        crate::jwt::JwtError::Expired => AuthError::TokenExpired,
        _ => AuthError::InvalidToken,
    })?;

    // Parse user ID
    let user_id = claims.sub.parse().map_err(|_| AuthError::InvalidToken)?;

    // Store user info in request extensions
    request.extensions_mut().insert(AuthUser {
        user_id,
        email: claims.email,
        is_admin: claims.is_admin,
    });

    Ok(next.run(request).await)
}

/// JWT authentication middleware with AppState access (S12-1d).
///
/// Validates JWT tokens and checks whether the token was issued before the
/// user's most recent password change — if so, rejects it.
pub async fn jwt_auth_middleware_with_state(
    State(state): State<AppState>,
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
            crate::jwt::extract_token_from_header(header).ok_or(AuthError::MissingToken)?
        }
        None => return Err(AuthError::MissingToken),
    };

    // Validate token
    let claims = validate_token(token, &state.jwt_config).map_err(|e| match e {
        crate::jwt::JwtError::Expired => AuthError::TokenExpired,
        _ => AuthError::InvalidToken,
    })?;

    // Parse user ID
    let user_id: uuid::Uuid = claims.sub.parse().map_err(|_| AuthError::InvalidToken)?;

    // Check if token was issued before most recent password change (S12-1d)
    let token_invalidated = if let Some(ref redis) = state.redis {
        // Try Redis first for fast path
        let key = format!("pw_changed:{}", user_id);
        match redis.get_i64(&key).await {
            Ok(Some(pw_changed_ts)) => claims.iat < pw_changed_ts,
            Ok(None) => {
                // Redis key doesn't exist — check DB as fallback
                check_password_changed_at_db(&state, user_id, claims.iat).await
            }
            Err(_) => {
                // Redis error — fall back to DB
                check_password_changed_at_db(&state, user_id, claims.iat).await
            }
        }
    } else {
        // No Redis — check DB directly
        check_password_changed_at_db(&state, user_id, claims.iat).await
    };

    if token_invalidated {
        return Err(AuthError::TokenExpired);
    }

    // Store user info in request extensions
    request.extensions_mut().insert(AuthUser {
        user_id,
        email: claims.email,
        is_admin: claims.is_admin,
    });

    Ok(next.run(request).await)
}

/// Check DB for password_changed_at and compare against token iat.
async fn check_password_changed_at_db(state: &AppState, user_id: uuid::Uuid, iat: i64) -> bool {
    let result = sqlx::query_as::<_, (Option<chrono::DateTime<chrono::Utc>>,)>(
        "SELECT password_changed_at FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(state.db.pool())
    .await;

    match result {
        Ok(Some((Some(pw_changed_at),))) => iat < pw_changed_at.timestamp(),
        _ => false,
    }
}
