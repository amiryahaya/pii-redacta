//! JWT Authentication for PII Redacta API
//!
//! Provides JWT token generation and validation for user authentication.

use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Minimum JWT secret length in bytes for security
const MIN_JWT_SECRET_LENGTH: usize = 32;

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Issued at timestamp
    pub iat: i64,
    /// Expiration timestamp
    pub exp: i64,
    /// Email
    pub email: String,
    /// Is admin
    pub is_admin: bool,
}

/// JWT configuration
#[derive(Clone)]
pub struct JwtConfig {
    /// Secret key for signing tokens
    secret: Arc<String>,
    /// Token expiration time in hours
    expiration_hours: i64,
}

impl JwtConfig {
    /// Create a new JWT configuration
    ///
    /// # Errors
    /// Returns `JwtError::SecretTooShort` if secret is less than 32 bytes
    pub fn new(secret: impl Into<String>, expiration_hours: i64) -> Result<Self, JwtError> {
        let secret = secret.into();
        if secret.len() < MIN_JWT_SECRET_LENGTH {
            return Err(JwtError::SecretTooShort {
                actual: secret.len(),
                minimum: MIN_JWT_SECRET_LENGTH,
            });
        }
        Ok(Self {
            secret: Arc::new(secret),
            expiration_hours,
        })
    }

    /// Get the secret key
    pub fn secret(&self) -> &str {
        &self.secret
    }
}

/// JWT error types
#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    #[error("Token has expired")]
    Expired,
    #[error("Invalid token")]
    Invalid,
    #[error("Token creation failed")]
    CreationFailed,
    #[error("JWT secret too short: got {actual} bytes, minimum {minimum} bytes required")]
    SecretTooShort { actual: usize, minimum: usize },
}

/// Generate a JWT token for a user
pub fn generate_token(
    user_id: Uuid,
    email: &str,
    is_admin: bool,
    config: &JwtConfig,
) -> Result<String, JwtError> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        iat: now.timestamp(),
        exp: (now
            + chrono::Duration::try_hours(config.expiration_hours)
                .expect("expiration_hours within valid range"))
        .timestamp(),
        email: email.to_string(),
        is_admin,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
    .map_err(|_| JwtError::CreationFailed)
}

/// Validate and decode a JWT token
pub fn validate_token(token: &str, config: &JwtConfig) -> Result<Claims, JwtError> {
    let validation = Validation::default();
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret.as_bytes()),
        &validation,
    )
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtError::Expired,
        _ => JwtError::Invalid,
    })?;

    Ok(token_data.claims)
}

/// Extract token from Authorization header
pub fn extract_token_from_header(auth_header: &str) -> Option<&str> {
    // Check for Bearer token format
    if let Some(token) = auth_header.strip_prefix("Bearer ") {
        return Some(token.trim());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_validate_token() {
        let config = JwtConfig::new("test-secret-key-at-least-32-bytes-long", 24).unwrap();
        let user_id = Uuid::new_v4();

        let token = generate_token(user_id, "test@example.com", false, &config).unwrap();
        assert!(!token.is_empty());

        let claims = validate_token(&token, &config).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.email, "test@example.com");
        assert!(!claims.is_admin);
    }

    #[test]
    fn test_invalid_token() {
        let config = JwtConfig::new("test-secret-key-at-least-32-bytes-long", 24).unwrap();
        let result = validate_token("invalid.token.here", &config);
        assert!(matches!(result, Err(JwtError::Invalid)));
    }

    #[test]
    fn test_extract_token_from_header() {
        assert_eq!(
            extract_token_from_header("Bearer my-token"),
            Some("my-token")
        );
        assert_eq!(
            extract_token_from_header("Bearer  my-token  "),
            Some("my-token")
        );
        assert_eq!(extract_token_from_header("Basic dXNlcjpwYXNz"), None);
        assert_eq!(extract_token_from_header(""), None);
    }

    #[test]
    fn test_secret_too_short() {
        let result = JwtConfig::new("short", 24);
        assert!(matches!(
            result,
            Err(JwtError::SecretTooShort {
                actual: 5,
                minimum: 32
            })
        ));
    }

    #[test]
    fn test_secret_minimum_length() {
        // Exactly 32 bytes should work
        let result = JwtConfig::new("this-is-exactly-32-bytes-long!!!", 24);
        assert!(result.is_ok());
    }
}
