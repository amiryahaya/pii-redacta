//! Request extractors for PII Redacta API

use uuid::Uuid;

/// Authenticated user extracted from JWT token
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
    pub is_admin: bool,
}

/// Admin user extracted after server-side admin verification (S12-2a)
#[derive(Debug, Clone)]
pub struct AdminUser {
    pub user_id: Uuid,
    pub email: String,
}
