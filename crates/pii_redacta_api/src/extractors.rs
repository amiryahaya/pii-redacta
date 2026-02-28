//! Request extractors for PII Redacta API

use uuid::Uuid;

/// Authenticated user extracted from JWT token
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
    pub is_admin: bool,
}
