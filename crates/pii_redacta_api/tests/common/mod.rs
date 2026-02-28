//! Test utilities for PII Redacta API integration tests
//!
//! This module provides shared utilities for integration tests including:
//! - Database connection management
//! - Test fixtures and factories
//! - HTTP client setup for API testing
//! - Database cleanup utilities

#![allow(dead_code)]

pub mod fixtures;

use axum::Router;
use pii_redacta_api::create_app_with_auth;
use pii_redacta_core::db::Database;
use std::sync::Arc;

/// Get database URL from environment or use default
pub fn database_url() -> String {
    std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://pii_redacta:pii_redacta_dev@localhost:5432/pii_redacta_test".to_string()
    })
}

/// Test JWT secret (32+ bytes)
pub fn test_jwt_secret() -> String {
    "test-secret-key-at-least-32-bytes-long-for-testing".to_string()
}

/// Test server secret for API keys (base64 encoded 32 bytes)
pub fn test_server_secret() -> String {
    "dGVzdC1zZWNyZXQtMzItYnl0ZXMtbG9uZy1rZXktZm9yLWhtYWM=".to_string()
}

/// Create a test database connection
///
/// Note: Migrations must be applied before running integration tests.
/// Use `./scripts/pods.sh migrate` or `sqlx migrate run`.
pub async fn setup_db() -> Arc<Database> {
    let db = Database::new(&database_url())
        .await
        .expect("Failed to connect to test database. Make sure PostgreSQL is running.");
    Arc::new(db)
}

/// Create a test app with authentication
pub async fn setup_app() -> Router {
    let db = setup_db().await;
    create_app_with_auth(db, &test_jwt_secret(), &test_server_secret(), None)
        .expect("Failed to create test app")
}

/// Create a test app with specific CORS origins
pub async fn setup_app_with_cors(origins: Vec<String>) -> Router {
    let db = setup_db().await;
    create_app_with_auth(db, &test_jwt_secret(), &test_server_secret(), Some(origins))
        .expect("Failed to create test app")
}

/// Test context that manages database state and cleanup
pub struct TestContext {
    pub db: Arc<Database>,
    pub created_user_ids: Vec<uuid::Uuid>,
    pub created_api_key_ids: Vec<uuid::Uuid>,
    pub created_tier_ids: Vec<uuid::Uuid>,
}

impl TestContext {
    /// Create a new test context
    pub async fn new() -> Self {
        let db = setup_db().await;
        Self {
            db,
            created_user_ids: Vec::new(),
            created_api_key_ids: Vec::new(),
            created_tier_ids: Vec::new(),
        }
    }

    /// Track a user for cleanup
    pub fn track_user(&mut self, user_id: uuid::Uuid) {
        self.created_user_ids.push(user_id);
    }

    /// Track an API key for cleanup
    pub fn track_api_key(&mut self, key_id: uuid::Uuid) {
        self.created_api_key_ids.push(key_id);
    }

    /// Track a tier for cleanup
    pub fn track_tier(&mut self, tier_id: uuid::Uuid) {
        self.created_tier_ids.push(tier_id);
    }

    /// Clean up all tracked resources
    pub async fn cleanup(&self) {
        // Clean up API keys first (foreign key constraints)
        for key_id in &self.created_api_key_ids {
            let _ = sqlx::query("DELETE FROM api_keys WHERE id = $1")
                .bind(key_id)
                .execute(self.db.pool())
                .await;
        }

        // Clean up subscriptions
        for user_id in &self.created_user_ids {
            let _ = sqlx::query("DELETE FROM subscriptions WHERE user_id = $1")
                .bind(user_id)
                .execute(self.db.pool())
                .await;
        }

        // Clean up users
        for user_id in &self.created_user_ids {
            let _ = sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(self.db.pool())
                .await;
        }

        // Clean up tiers
        for tier_id in &self.created_tier_ids {
            let _ = sqlx::query("DELETE FROM tiers WHERE id = $1")
                .bind(tier_id)
                .execute(self.db.pool())
                .await;
        }
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        // Try to clean up synchronously if possible
        // Note: This is best-effort; tests should call cleanup() explicitly
    }
}

/// Helper macro to run a test with a database context that cleans up after
#[macro_export]
macro_rules! with_test_context {
    ($name:ident, $body:block) => {
        #[tokio::test]
        async fn $name() {
            let mut ctx = $crate::common::TestContext::new().await;
            let result = async { $body }.await;
            ctx.cleanup().await;
            result
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_url() {
        let url = database_url();
        assert!(url.contains("postgres://"));
        assert!(url.contains("pii_redacta_test"));
    }

    #[test]
    fn test_jwt_secret_length() {
        assert!(test_jwt_secret().len() >= 32);
    }

    #[test]
    fn test_server_secret_valid_base64() {
        let decoded = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            test_server_secret(),
        );
        assert!(decoded.is_ok());
        // The decoded secret should be at least 32 bytes
        assert!(decoded.unwrap().len() >= 32);
    }
}
