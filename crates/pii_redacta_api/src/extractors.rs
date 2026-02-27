//! Request extractors for PII Redacta API

use crate::jwt::Claims;
use axum::{
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use serde_json::json;
use uuid::Uuid;

/// Authenticated user extracted from JWT token
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
    pub is_admin: bool,
}

impl AuthUser {
    /// Create from JWT claims
    pub fn from_claims(claims: Claims) -> Result<Self, AuthExtractorError> {
        let user_id = claims
            .sub
            .parse::<Uuid>()
            .map_err(|_| AuthExtractorError::InvalidToken)?;

        Ok(Self {
            user_id,
            email: claims.email,
            is_admin: claims.is_admin,
        })
    }
}

/// Error type for auth extraction
#[derive(Debug)]
pub enum AuthExtractorError {
    MissingToken,
    InvalidToken,
}

impl IntoResponse for AuthExtractorError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthExtractorError::MissingToken => {
                (StatusCode::UNAUTHORIZED, "Missing authorization token")
            }
            AuthExtractorError::InvalidToken => {
                (StatusCode::UNAUTHORIZED, "Invalid or expired token")
            }
        };

        let body = json!({
            "error": {
                "code": status.as_u16(),
                "message": message,
            }
        });

        (status, axum::Json(body)).into_response()
    }
}

/// Extract AuthUser from request extensions
///
/// This is used by handlers that have the jwt_auth_middleware applied.
/// The middleware injects the AuthUser into request extensions.
pub fn get_auth_user_from_extensions(parts: &Parts) -> Option<AuthUser> {
    parts.extensions.get::<AuthUser>().cloned()
}
