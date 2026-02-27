//! ApiKeyManager Integration Tests
//!
//! These tests require PostgreSQL to be running.
//! Run with: cargo test --test api_key_manager_test

use pii_redacta_core::db::api_key_manager::{ApiKeyEnvironment, ApiKeyManager};
use pii_redacta_core::db::Database;
use std::sync::Arc;

/// Get database URL from environment or use default
fn database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://pii_redacta:pii_redacta_dev@localhost:5432/pii_redacta_dev".to_string()
    })
}

/// Test server secret (32 bytes base64 encoded)
/// This is a test secret - do not use in production!
fn test_server_secret() -> String {
    "dGVzdC1zZWNyZXQtMzItYnl0ZXMtbG9uZy1rZXktZm9yLWhtYWM=".to_string()
}

/// Helper to create a test database connection
async fn setup_db() -> Arc<Database> {
    let db = Database::new(&database_url())
        .await
        .expect("Failed to connect to database");
    Arc::new(db)
}

/// Helper to create an ApiKeyManager
async fn setup_api_key_manager() -> ApiKeyManager {
    let db = setup_db().await;
    ApiKeyManager::new(db, &test_server_secret()).expect("Failed to create ApiKeyManager")
}

/// Create a test user and return the user ID
async fn create_test_user(db: &Database) -> uuid::Uuid {
    let user_id = uuid::Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, created_at, updated_at)
        VALUES ($1, $2, $3, NOW(), NOW())
        "#,
    )
    .bind(user_id)
    .bind(format!("test-{}@example.com", user_id))
    .bind("test-hash")
    .execute(db.pool())
    .await
    .expect("Failed to create test user");

    user_id
}

/// Clean up test user and their keys
async fn cleanup_test_user(db: &Database, user_id: uuid::Uuid) {
    // Delete API keys first (cascade will handle it, but let's be explicit)
    sqlx::query("DELETE FROM api_keys WHERE user_id = $1")
        .bind(user_id)
        .execute(db.pool())
        .await
        .ok();

    // Delete user
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(db.pool())
        .await
        .ok();
}

#[tokio::test]
async fn test_generate_live_api_key() {
    let manager = setup_api_key_manager().await;
    let db = setup_db().await;
    let user_id = create_test_user(&db).await;

    // Generate a live API key
    let key = manager
        .generate_key(user_id, "My Test Key", ApiKeyEnvironment::Live, None)
        .await
        .expect("Should generate key");

    // Verify key format
    assert!(key.full_key.starts_with("pii_live_"));
    assert_eq!(key.environment, ApiKeyEnvironment::Live);
    assert_eq!(key.user_id, user_id);
    assert_eq!(key.prefix.len(), 8);
    assert!(key.expires_at.is_none());

    // Key should be: pii_live_{prefix}_{64_char_secret}
    let parts: Vec<&str> = key.full_key.split('_').collect();
    assert_eq!(parts.len(), 4);
    assert_eq!(parts[0], "pii");
    assert_eq!(parts[1], "live");
    assert_eq!(parts[2].len(), 8);
    assert_eq!(parts[3].len(), 64); // 64 hex chars = 32 bytes

    cleanup_test_user(&db, user_id).await;
}

#[tokio::test]
async fn test_generate_test_api_key() {
    let manager = setup_api_key_manager().await;
    let db = setup_db().await;
    let user_id = create_test_user(&db).await;

    let key = manager
        .generate_key(
            user_id,
            "Test Environment Key",
            ApiKeyEnvironment::Test,
            None,
        )
        .await
        .expect("Should generate key");

    assert!(key.full_key.starts_with("pii_test_"));
    assert_eq!(key.environment, ApiKeyEnvironment::Test);

    cleanup_test_user(&db, user_id).await;
}

#[tokio::test]
async fn test_generate_key_with_expiration() {
    let manager = setup_api_key_manager().await;
    let db = setup_db().await;
    let user_id = create_test_user(&db).await;

    let expiration = chrono::Utc::now() + chrono::Duration::days(30);
    let key = manager
        .generate_key(
            user_id,
            "Expiring Key",
            ApiKeyEnvironment::Live,
            Some(expiration),
        )
        .await
        .expect("Should generate key");

    assert!(key.expires_at.is_some());
    let expires = key.expires_at.unwrap();
    // Allow 1 second tolerance
    let diff = (expires - expiration).num_seconds().abs();
    assert!(diff < 1);

    cleanup_test_user(&db, user_id).await;
}

#[tokio::test]
async fn test_validate_key_success() {
    let manager = setup_api_key_manager().await;
    let db = setup_db().await;
    let user_id = create_test_user(&db).await;

    // Generate key
    let key = manager
        .generate_key(user_id, "Validation Test", ApiKeyEnvironment::Live, None)
        .await
        .expect("Should generate key");

    // Validate it
    let validated = manager
        .validate_key(&key.full_key)
        .await
        .expect("Should validate key");

    assert_eq!(validated.user_id, user_id);
    assert!(matches!(validated.environment, ApiKeyEnvironment::Live));
    assert_eq!(validated.api_key.name, "Validation Test");
    assert!(validated.api_key.is_active);

    cleanup_test_user(&db, user_id).await;
}

#[tokio::test]
async fn test_validate_key_not_found() {
    let manager = setup_api_key_manager().await;

    // Create a key with valid format but that doesn't exist in DB
    let fake_key =
        "pii_live_a1b2c3d4_e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2";

    let result = manager.validate_key(fake_key).await;
    assert!(result.is_err());

    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("not found") || err_msg.contains("revoked"));
}

#[tokio::test]
async fn test_validate_key_invalid_format() {
    let manager = setup_api_key_manager().await;

    // Invalid format
    let result = manager.validate_key("invalid-key").await;
    assert!(result.is_err());

    let err_msg = format!("{}", result.unwrap_err());
    // Accept either InvalidFormat or NotFound errors
    assert!(
        err_msg.to_lowercase().contains("invalid")
            || err_msg.to_lowercase().contains("not found")
            || err_msg.to_lowercase().contains("revoked"),
        "Error message was: {}",
        err_msg
    );
}

#[tokio::test]
async fn test_validate_key_expired() {
    let manager = setup_api_key_manager().await;
    let db = setup_db().await;
    let user_id = create_test_user(&db).await;

    // Create an already-expired key
    let expiration = chrono::Utc::now() - chrono::Duration::days(1);
    let key = manager
        .generate_key(
            user_id,
            "Expired Key",
            ApiKeyEnvironment::Live,
            Some(expiration),
        )
        .await
        .expect("Should generate key");

    // Try to validate - should fail with expired error
    let result = manager.validate_key(&key.full_key).await;
    assert!(result.is_err());

    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("expired") || err_msg.contains("not found"));

    cleanup_test_user(&db, user_id).await;
}

#[tokio::test]
async fn test_validate_key_updates_last_used() {
    let manager = setup_api_key_manager().await;
    let db = setup_db().await;
    let user_id = create_test_user(&db).await;

    // Generate key
    let key = manager
        .generate_key(user_id, "Last Used Test", ApiKeyEnvironment::Live, None)
        .await
        .expect("Should generate key");

    // First validation
    let validated1 = manager
        .validate_key(&key.full_key)
        .await
        .expect("Should validate key");

    let last_used_1 = validated1.api_key.last_used_at;

    // Small delay
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Second validation
    let validated2 = manager
        .validate_key(&key.full_key)
        .await
        .expect("Should validate key again");

    let last_used_2 = validated2.api_key.last_used_at;

    // last_used should be updated
    assert!(last_used_2 > last_used_1);

    cleanup_test_user(&db, user_id).await;
}

#[tokio::test]
async fn test_revoke_key() {
    let manager = setup_api_key_manager().await;
    let db = setup_db().await;
    let user_id = create_test_user(&db).await;

    // Generate key
    let key = manager
        .generate_key(user_id, "To Be Revoked", ApiKeyEnvironment::Live, None)
        .await
        .expect("Should generate key");

    // Validate should work initially
    let validated = manager.validate_key(&key.full_key).await;
    assert!(validated.is_ok());

    // Get the key ID
    let keys = manager
        .list_user_keys(user_id)
        .await
        .expect("Should list keys");
    let key_id = keys[0].id;

    // Revoke the key
    let revoked = manager
        .revoke_key(key_id, user_id, Some("Test revocation"))
        .await
        .expect("Should revoke key");

    assert!(!revoked.is_active);
    assert!(revoked.revoked_at.is_some());
    assert_eq!(revoked.revoked_reason.as_deref(), Some("Test revocation"));

    // Validation should now fail
    let result = manager.validate_key(&key.full_key).await;
    assert!(result.is_err());

    cleanup_test_user(&db, user_id).await;
}

#[tokio::test]
async fn test_list_user_keys() {
    let manager = setup_api_key_manager().await;
    let db = setup_db().await;
    let user_id = create_test_user(&db).await;

    // Initially no keys
    let keys = manager
        .list_user_keys(user_id)
        .await
        .expect("Should list keys");
    assert!(keys.is_empty());

    // Generate some keys
    manager
        .generate_key(user_id, "Key 1", ApiKeyEnvironment::Live, None)
        .await
        .unwrap();
    manager
        .generate_key(user_id, "Key 2", ApiKeyEnvironment::Test, None)
        .await
        .unwrap();
    manager
        .generate_key(user_id, "Key 3", ApiKeyEnvironment::Live, None)
        .await
        .unwrap();

    let keys = manager
        .list_user_keys(user_id)
        .await
        .expect("Should list keys");
    assert_eq!(keys.len(), 3);

    // Should be sorted by created_at DESC (newest first)
    assert_eq!(keys[0].name, "Key 3");
    assert_eq!(keys[1].name, "Key 2");
    assert_eq!(keys[2].name, "Key 1");

    cleanup_test_user(&db, user_id).await;
}

#[tokio::test]
async fn test_count_user_keys() {
    let manager = setup_api_key_manager().await;
    let db = setup_db().await;
    let user_id = create_test_user(&db).await;

    let count = manager
        .count_user_keys(user_id)
        .await
        .expect("Should count keys");
    assert_eq!(count, 0);

    // Add keys
    manager
        .generate_key(user_id, "Key 1", ApiKeyEnvironment::Live, None)
        .await
        .unwrap();
    manager
        .generate_key(user_id, "Key 2", ApiKeyEnvironment::Live, None)
        .await
        .unwrap();

    let count = manager
        .count_user_keys(user_id)
        .await
        .expect("Should count keys");
    assert_eq!(count, 2);

    cleanup_test_user(&db, user_id).await;
}

#[tokio::test]
async fn test_get_key() {
    let manager = setup_api_key_manager().await;
    let db = setup_db().await;
    let user_id = create_test_user(&db).await;

    // Generate key
    manager
        .generate_key(user_id, "Get Test", ApiKeyEnvironment::Live, None)
        .await
        .unwrap();

    let keys = manager.list_user_keys(user_id).await.unwrap();
    let key_id = keys[0].id;

    // Get specific key
    let key = manager
        .get_key(key_id, user_id)
        .await
        .expect("Should get key");
    assert_eq!(key.name, "Get Test");

    // Get non-existent key
    let fake_id = uuid::Uuid::new_v4();
    let result = manager.get_key(fake_id, user_id).await;
    assert!(result.is_err());

    // Get key belonging to different user
    let user_id_2 = create_test_user(&db).await;
    let result = manager.get_key(key_id, user_id_2).await;
    assert!(result.is_err());

    cleanup_test_user(&db, user_id).await;
    cleanup_test_user(&db, user_id_2).await;
}

#[tokio::test]
async fn test_validate_key_readonly() {
    let manager = setup_api_key_manager().await;
    let db = setup_db().await;
    let user_id = create_test_user(&db).await;

    // Generate key
    let key = manager
        .generate_key(user_id, "Readonly Test", ApiKeyEnvironment::Live, None)
        .await
        .expect("Should generate key");

    // Validate with readonly (should not update last_used_at)
    let validated1 = manager
        .validate_key_readonly(&key.full_key)
        .await
        .expect("Should validate key");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let validated2 = manager
        .validate_key_readonly(&key.full_key)
        .await
        .expect("Should validate key again");

    // last_used_at should be the same (not updated)
    assert_eq!(
        validated1.api_key.last_used_at,
        validated2.api_key.last_used_at
    );

    cleanup_test_user(&db, user_id).await;
}

#[tokio::test]
async fn test_revoke_wrong_user() {
    let manager = setup_api_key_manager().await;
    let db = setup_db().await;
    let user_id_1 = create_test_user(&db).await;
    let user_id_2 = create_test_user(&db).await;

    // Generate key for user 1
    manager
        .generate_key(user_id_1, "User 1 Key", ApiKeyEnvironment::Live, None)
        .await
        .unwrap();
    let keys = manager.list_user_keys(user_id_1).await.unwrap();
    let key_id = keys[0].id;

    // Try to revoke with user 2
    let result = manager.revoke_key(key_id, user_id_2, Some("Hacked!")).await;
    assert!(result.is_err());

    // Key should still be valid
    let keys = manager.list_user_keys(user_id_1).await.unwrap();
    assert_eq!(keys.len(), 1);
    assert!(keys[0].is_active);

    cleanup_test_user(&db, user_id_1).await;
    cleanup_test_user(&db, user_id_2).await;
}

#[tokio::test]
async fn test_keys_are_unique() {
    let manager = setup_api_key_manager().await;
    let db = setup_db().await;
    let user_id = create_test_user(&db).await;

    // Generate multiple keys
    let key1 = manager
        .generate_key(user_id, "Key 1", ApiKeyEnvironment::Live, None)
        .await
        .unwrap();
    let key2 = manager
        .generate_key(user_id, "Key 2", ApiKeyEnvironment::Live, None)
        .await
        .unwrap();
    let key3 = manager
        .generate_key(user_id, "Key 3", ApiKeyEnvironment::Live, None)
        .await
        .unwrap();

    // All keys should be different
    assert_ne!(key1.full_key, key2.full_key);
    assert_ne!(key2.full_key, key3.full_key);
    assert_ne!(key1.prefix, key2.prefix);
    assert_ne!(key2.prefix, key3.prefix);

    cleanup_test_user(&db, user_id).await;
}
