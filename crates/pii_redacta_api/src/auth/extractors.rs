//! Request extractors for authentication
//!
//! Provides functions for extracting and validating
//! authentication information from incoming requests.

use super::{AuthError, AuthState, AuthenticatedUser};
use axum::http::HeaderMap;

/// Extract and validate API key from request, returning full user info
///
/// # Usage in handlers
///
/// ```rust,ignore
/// async fn handler(
///     State(state): State<Arc<AuthState>>,
///     headers: HeaderMap,
/// ) -> Result<impl IntoResponse, AuthError> {
///     let auth = extract_api_auth(&state, &headers).await?;
///     Ok(format!("Hello, user {}!", auth.user_id))
/// }
/// ```
pub async fn extract_api_auth(
    state: &AuthState,
    headers: &HeaderMap,
) -> Result<AuthenticatedUser, AuthError> {
    // Extract API key from headers
    let api_key = extract_api_key(headers)?;

    // Validate the API key
    let validated = state
        .api_key_manager
        .validate_key(&api_key)
        .await
        .map_err(AuthError::ApiKey)?;

    // Get user's tier information (S9-R3-02: ORDER BY + LIMIT 1)
    let subscription = sqlx::query_as::<_, (uuid::Uuid,)>(
        r#"
        SELECT tier_id FROM subscriptions
        WHERE user_id = $1 AND status IN ('trial', 'active', 'past_due')
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(validated.user_id)
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
            // Fallback to trial tier if no subscription found
            state
                .tier_manager
                .get_by_name("trial")
                .await
                .map_err(AuthError::TierManager)?
        }
    };

    Ok(AuthenticatedUser {
        user_id: validated.user_id,
        api_key_id: validated.api_key.id,
        environment: validated.environment,
        tier_name: tier.name,
        tier_limits: tier.limits.0,
        tier_features: tier.features.0,
    })
}

/// Extract and validate API key without fetching tier info (lighter weight)
pub async fn extract_simple_auth(
    state: &AuthState,
    headers: &HeaderMap,
) -> Result<SimpleAuth, AuthError> {
    let api_key = extract_api_key(headers)?;

    let validated = state
        .api_key_manager
        .validate_key(&api_key)
        .await
        .map_err(AuthError::ApiKey)?;

    Ok(SimpleAuth {
        user_id: validated.user_id,
        api_key_id: validated.api_key.id,
        environment: validated.environment,
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

/// Try to extract API key, returning None if not present or invalid
pub async fn try_extract_auth(state: &AuthState, headers: &HeaderMap) -> Option<AuthenticatedUser> {
    // Try to extract API key
    let api_key = match extract_api_key(headers) {
        Ok(key) => key,
        Err(_) => return None,
    };

    // Validate the API key
    let validated = match state.api_key_manager.validate_key(&api_key).await {
        Ok(v) => v,
        Err(_) => return None,
    };

    // Get user's tier information (S9-R3-02: ORDER BY + LIMIT 1)
    let subscription = sqlx::query_as::<_, (uuid::Uuid,)>(
        r#"
        SELECT tier_id FROM subscriptions
        WHERE user_id = $1 AND status IN ('trial', 'active', 'past_due')
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(validated.user_id)
    .fetch_optional(state.api_key_manager.pool())
    .await
    .ok()?;

    let tier = match subscription {
        Some((tier_id,)) => match state.tier_manager.get_by_id(tier_id).await {
            Ok(t) => t,
            Err(_) => return None,
        },
        None => match state.tier_manager.get_by_name("trial").await {
            Ok(t) => t,
            Err(_) => return None,
        },
    };

    Some(AuthenticatedUser {
        user_id: validated.user_id,
        api_key_id: validated.api_key.id,
        environment: validated.environment,
        tier_name: tier.name,
        tier_limits: tier.limits.0,
        tier_features: tier.features.0,
    })
}

/// Extract API key from Authorization header
///
/// Supports:
/// - `Authorization: Bearer <api_key>`
/// - `Authorization: <api_key>` (legacy fallback)
pub fn extract_api_key(headers: &HeaderMap) -> Result<String, AuthError> {
    let auth_header = headers
        .get("Authorization")
        .ok_or(AuthError::MissingApiKey)?
        .to_str()
        .map_err(|_| AuthError::InvalidApiKeyFormat)?;

    // Try Bearer token format first
    if let Some(token) = auth_header.strip_prefix("Bearer ") {
        return Ok(token.trim().to_string());
    }

    // Fallback: use entire header value (for backward compatibility)
    // But only if it looks like our API key format
    if auth_header.starts_with("pii_") {
        return Ok(auth_header.to_string());
    }

    Err(AuthError::InvalidApiKeyFormat)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_extract_api_key_bearer_format() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_static("Bearer pii_live_abc123_def456"),
        );

        let key = extract_api_key(&headers).unwrap();
        assert_eq!(key, "pii_live_abc123_def456");
    }

    #[test]
    fn test_extract_api_key_direct_format() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_static("pii_live_abc123_def456"),
        );

        let key = extract_api_key(&headers).unwrap();
        assert_eq!(key, "pii_live_abc123_def456");
    }

    #[test]
    fn test_extract_api_key_missing() {
        let headers = HeaderMap::new();

        let result = extract_api_key(&headers);
        assert!(matches!(result, Err(AuthError::MissingApiKey)));
    }

    #[test]
    fn test_extract_api_key_invalid_format() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_static("Basic dXNlcjpwYXNz"),
        );

        let result = extract_api_key(&headers);
        assert!(matches!(result, Err(AuthError::InvalidApiKeyFormat)));
    }

    #[test]
    fn test_extract_api_key_with_whitespace() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_static("Bearer  pii_live_abc123_def456  "),
        );

        let key = extract_api_key(&headers).unwrap();
        assert_eq!(key, "pii_live_abc123_def456");
    }
}
