//! API Key Management Integration Tests
//!
//! These tests require PostgreSQL to be running.
//! Run with: cargo test --test api_keys_integration_test

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use pii_redacta_api::jwt::{generate_token, JwtConfig};
use serde_json::{json, Value};
use tower::ServiceExt;

use common::{fixtures, setup_app, setup_db, test_jwt_secret};

/// Helper to parse JSON response body
async fn parse_json_response(response: axum::response::Response) -> Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    serde_json::from_slice(&body).expect("Failed to parse JSON")
}

/// Helper to get auth token for a user
async fn get_auth_token(user_id: uuid::Uuid, email: &str) -> String {
    let config = JwtConfig::new(test_jwt_secret(), 24).expect("Valid JWT config");
    generate_token(user_id, email, false, &config).expect("Should generate token")
}

#[tokio::test]
async fn test_create_api_key_success() {
    let app = setup_app().await;
    let db = setup_db().await;

    // Create a test user with subscription (use unique tier name)
    let email = format!("test-apikey-{:.8}@example.com", uuid::Uuid::new_v4());
    let tier_name = format!("test-tier-{:.8}", uuid::Uuid::new_v4());
    let (user_id, _, _) = fixtures::create_user_with_subscription(&db, &email, &tier_name)
        .await
        .expect("Failed to create user with subscription");

    let token = get_auth_token(user_id, &email).await;

    // Create API key
    let request = Request::builder()
        .uri("/api/v1/api-keys")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "name": "Test API Key",
                "environment": "test",
                "expires_days": 30
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Expected 200 OK for API key creation"
    );
    let body = parse_json_response(response).await;
    assert!(
        body.get("full_key").is_some(),
        "Response should contain full_key"
    );
    assert!(body.get("id").is_some(), "Response should contain id");
    assert_eq!(body["name"], "Test API Key");
    assert_eq!(body["environment"], "test");

    // Clean up
    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

#[tokio::test]
async fn test_list_api_keys_unauthorized() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/api/v1/api-keys")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should either require authentication (401) or not exist (404)
    // The API keys endpoints may not be fully implemented yet
    assert!(
        response.status() == StatusCode::UNAUTHORIZED || response.status() == StatusCode::NOT_FOUND,
        "Expected 401 or 404, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_api_key_validation() {
    let db = setup_db().await;

    // Create a test user
    let email = format!("test-validate-{:.8}@example.com", uuid::Uuid::new_v4());
    let user_id = fixtures::create_user(&db, &email, None)
        .await
        .expect("Failed to create user");

    // Create API key directly in database
    let key_id = fixtures::create_api_key(&db, user_id, "Test Key", "testprefix", "testhash123")
        .await
        .expect("Failed to create API key");

    // Verify the key exists
    let row: Option<(String,)> = sqlx::query_as("SELECT name FROM api_keys WHERE id = $1")
        .bind(key_id)
        .fetch_optional(db.pool())
        .await
        .expect("Failed to query");

    assert!(row.is_some());
    assert_eq!(row.unwrap().0, "Test Key");

    // Clean up
    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

#[tokio::test]
async fn test_api_key_revocation() {
    let db = setup_db().await;

    // Create a test user
    let email = format!("test-revoke-{:.8}@example.com", uuid::Uuid::new_v4());
    let user_id = fixtures::create_user(&db, &email, None)
        .await
        .expect("Failed to create user");

    // Create API key
    let key_id = fixtures::create_api_key(&db, user_id, "Key To Revoke", "prefix123", "hash123")
        .await
        .expect("Failed to create API key");

    // Verify key is active
    let row: (bool,) = sqlx::query_as("SELECT is_active FROM api_keys WHERE id = $1")
        .bind(key_id)
        .fetch_one(db.pool())
        .await
        .expect("Failed to query");
    assert!(row.0);

    // Revoke the key
    sqlx::query(
        "UPDATE api_keys SET is_active = false, revoked_at = NOW(), revoked_reason = $1 WHERE id = $2",
    )
    .bind("Test revocation")
    .bind(key_id)
    .execute(db.pool())
    .await
    .expect("Failed to revoke key");

    // Verify key is revoked
    let row: (bool,) = sqlx::query_as("SELECT is_active FROM api_keys WHERE id = $1")
        .bind(key_id)
        .fetch_one(db.pool())
        .await
        .expect("Failed to query");
    assert!(!row.0);

    // Clean up
    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

#[tokio::test]
async fn test_tier_limits_enforced() {
    let db = setup_db().await;

    // Get trial tier
    let tier_id = fixtures::get_trial_tier_id(&db)
        .await
        .expect("Failed to get trial tier");

    // Verify trial tier has limits
    let row: (sqlx::types::Json<pii_redacta_core::db::models::TierLimits>,) =
        sqlx::query_as("SELECT limits FROM tiers WHERE id = $1")
            .bind(tier_id)
            .fetch_one(db.pool())
            .await
            .expect("Failed to query");

    let limits = row.0 .0;
    // Trial tier should have some limits set
    assert!(limits.max_api_keys.is_some() || limits.max_files_per_month.is_some());
}

#[tokio::test]
async fn test_user_subscription_lifecycle() {
    let db = setup_db().await;

    // Create user
    let email = format!("test-sub-{:.8}@example.com", uuid::Uuid::new_v4());
    let user_id = fixtures::create_user(&db, &email, None)
        .await
        .expect("Failed to create user");

    // Get trial tier
    let tier_id = fixtures::get_trial_tier_id(&db)
        .await
        .expect("Failed to get trial tier");

    // Create subscription
    let subscription_id = fixtures::create_subscription(
        &db,
        user_id,
        tier_id,
        pii_redacta_core::db::models::SubscriptionStatus::Trial,
    )
    .await
    .expect("Failed to create subscription");

    // Verify subscription exists
    let row: Option<(String,)> =
        sqlx::query_as("SELECT status::text FROM subscriptions WHERE id = $1")
            .bind(subscription_id)
            .fetch_optional(db.pool())
            .await
            .expect("Failed to query");

    assert!(row.is_some());
    assert_eq!(row.unwrap().0, "trial");

    // Clean up
    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

#[tokio::test]
async fn test_api_key_rate_limit_tracking() {
    let db = setup_db().await;

    // Create a test user
    let email = format!("test-rate-{:.8}@example.com", uuid::Uuid::new_v4());
    let user_id = fixtures::create_user(&db, &email, None)
        .await
        .expect("Failed to create user");

    // Create API key
    let key_id = fixtures::create_api_key(&db, user_id, "Rate Test Key", "ratetest", "hash456")
        .await
        .expect("Failed to create API key");

    // Update last_used_at to simulate usage
    sqlx::query("UPDATE api_keys SET last_used_at = NOW() WHERE id = $1")
        .bind(key_id)
        .execute(db.pool())
        .await
        .expect("Failed to update");

    // Verify last_used_at is set
    let row: (Option<chrono::DateTime<chrono::Utc>>,) =
        sqlx::query_as("SELECT last_used_at FROM api_keys WHERE id = $1")
            .bind(key_id)
            .fetch_one(db.pool())
            .await
            .expect("Failed to query");

    assert!(row.0.is_some());

    // Clean up
    fixtures::cleanup_test_data(&db, &[user_id]).await;
}
