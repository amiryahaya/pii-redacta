//! Integration tests for Playground endpoints
//!
//! Sprint 13: Authenticated Playground
//!
//! Run with: cargo test --test playground_test

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

// ============================================================================
// Auth Rejection Tests
// ============================================================================

#[tokio::test]
async fn test_playground_text_requires_auth() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/api/v1/playground/text")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({ "text": "My email is test@example.com" }).to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_playground_file_requires_auth() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/api/v1/playground/file")
        .method("POST")
        .header("Content-Type", "multipart/form-data; boundary=test")
        .body(Body::from("--test\r\n\r\n--test--"))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_playground_history_requires_auth() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/api/v1/playground/history")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ============================================================================
// Text Endpoint Tests
// ============================================================================

#[tokio::test]
async fn test_playground_text_success() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("pg-text-{:.8}@example.com", uuid::Uuid::new_v4());
    let tier_name = format!("pg-tier-{:.8}", uuid::Uuid::new_v4());
    let (user_id, _, _) = fixtures::create_user_with_subscription(&db, &email, &tier_name)
        .await
        .expect("Failed to create user");

    let token = get_auth_token(user_id, &email).await;

    let request = Request::builder()
        .uri("/api/v1/playground/text")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({ "text": "My email is test@example.com" }).to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_json_response(response).await;
    assert!(body.get("entities").is_some());
    assert!(body.get("processingTimeMs").is_some());
    assert!(body.get("textLength").is_some());
    assert!(body.get("dailyUsage").is_some());

    let entities = body["entities"].as_array().unwrap();
    assert!(!entities.is_empty(), "Should detect at least one entity");

    let usage = &body["dailyUsage"];
    assert!(usage.get("usedToday").is_some());
    assert!(usage.get("dailyLimit").is_some());

    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

#[tokio::test]
async fn test_playground_text_with_redaction() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("pg-redact-{:.8}@example.com", uuid::Uuid::new_v4());
    let tier_name = format!("pg-tier-{:.8}", uuid::Uuid::new_v4());
    let (user_id, _, _) = fixtures::create_user_with_subscription(&db, &email, &tier_name)
        .await
        .expect("Failed to create user");

    let token = get_auth_token(user_id, &email).await;

    let request = Request::builder()
        .uri("/api/v1/playground/text")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({ "text": "Contact me at john@example.com", "redact": true }).to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_json_response(response).await;
    assert!(body.get("redactedText").is_some());
    let redacted = body["redactedText"].as_str().unwrap();
    assert!(
        !redacted.contains("john@example.com"),
        "Redacted text should not contain original email"
    );

    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

#[tokio::test]
async fn test_playground_text_empty_rejected() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("pg-empty-{:.8}@example.com", uuid::Uuid::new_v4());
    let tier_name = format!("pg-tier-{:.8}", uuid::Uuid::new_v4());
    let (user_id, _, _) = fixtures::create_user_with_subscription(&db, &email, &tier_name)
        .await
        .expect("Failed to create user");

    let token = get_auth_token(user_id, &email).await;

    let request = Request::builder()
        .uri("/api/v1/playground/text")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(json!({ "text": "" }).to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

#[tokio::test]
async fn test_playground_text_too_long_rejected() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("pg-long-{:.8}@example.com", uuid::Uuid::new_v4());
    let tier_name = format!("pg-tier-{:.8}", uuid::Uuid::new_v4());
    let (user_id, _, _) = fixtures::create_user_with_subscription(&db, &email, &tier_name)
        .await
        .expect("Failed to create user");

    let token = get_auth_token(user_id, &email).await;

    // Create text > 1MB
    let long_text = "x".repeat(1_000_001);

    let request = Request::builder()
        .uri("/api/v1/playground/text")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(json!({ "text": long_text }).to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);

    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

// ============================================================================
// File Upload Tests
// ============================================================================

#[tokio::test]
async fn test_playground_file_upload_success() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("pg-file-{:.8}@example.com", uuid::Uuid::new_v4());
    let tier_name = format!("pg-tier-{:.8}", uuid::Uuid::new_v4());
    let (user_id, _, _) = fixtures::create_user_with_subscription(&db, &email, &tier_name)
        .await
        .expect("Failed to create user");

    let token = get_auth_token(user_id, &email).await;

    // Construct a multipart body with a text file containing PII
    let boundary = "----TestBoundary12345";
    let file_content = "Contact me at john@example.com or call +60123456789";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\nContent-Type: text/plain\r\n\r\n{file_content}\r\n--{boundary}--\r\n"
    );

    let request = Request::builder()
        .uri("/api/v1/playground/file")
        .method("POST")
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_json_response(response).await;
    assert!(body.get("entities").is_some());
    assert!(body.get("processingTimeMs").is_some());
    assert!(body.get("textLength").is_some());
    assert!(body.get("dailyUsage").is_some());

    let entities = body["entities"].as_array().unwrap();
    assert!(!entities.is_empty(), "Should detect at least one entity");

    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

#[tokio::test]
async fn test_playground_file_upload_with_redact() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("pg-filerd-{:.8}@example.com", uuid::Uuid::new_v4());
    let tier_name = format!("pg-tier-{:.8}", uuid::Uuid::new_v4());
    let (user_id, _, _) = fixtures::create_user_with_subscription(&db, &email, &tier_name)
        .await
        .expect("Failed to create user");

    let token = get_auth_token(user_id, &email).await;

    let boundary = "----TestBoundary67890";
    let file_content = "Email: secret@example.com";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"pii.txt\"\r\nContent-Type: text/plain\r\n\r\n{file_content}\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"redact\"\r\n\r\ntrue\r\n--{boundary}--\r\n"
    );

    let request = Request::builder()
        .uri("/api/v1/playground/file")
        .method("POST")
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_json_response(response).await;
    assert!(body.get("redactedText").is_some());
    let redacted = body["redactedText"].as_str().unwrap();
    assert!(
        !redacted.contains("secret@example.com"),
        "Redacted text should not contain original email"
    );

    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

// ============================================================================
// History Tests
// ============================================================================

#[tokio::test]
async fn test_playground_history_returns_entries() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("pg-hist-{:.8}@example.com", uuid::Uuid::new_v4());
    let tier_name = format!("pg-tier-{:.8}", uuid::Uuid::new_v4());
    let (user_id, _, _) = fixtures::create_user_with_subscription(&db, &email, &tier_name)
        .await
        .expect("Failed to create user");

    let token = get_auth_token(user_id, &email).await;

    // Submit a playground request first (usage is recorded synchronously)
    let submit_request = Request::builder()
        .uri("/api/v1/playground/text")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({ "text": "My email is test@example.com" }).to_string(),
        ))
        .unwrap();

    let submit_response = app.clone().oneshot(submit_request).await.unwrap();
    assert_eq!(submit_response.status(), StatusCode::OK);

    // Fetch history — no sleep needed, usage was recorded synchronously (M6 fix)
    let history_request = Request::builder()
        .uri("/api/v1/playground/history")
        .method("GET")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let history_response = app.oneshot(history_request).await.unwrap();
    assert_eq!(history_response.status(), StatusCode::OK);

    let body = parse_json_response(history_response).await;
    assert!(body.is_array());
    let entries = body.as_array().unwrap();
    assert!(
        !entries.is_empty(),
        "History should contain the submitted entry"
    );

    let first = &entries[0];
    assert!(first.get("id").is_some());
    assert!(first.get("requestType").is_some());
    assert!(first.get("success").is_some());
    assert!(first.get("createdAt").is_some());

    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

// ============================================================================
// Usage Recording Test
// ============================================================================

#[tokio::test]
async fn test_playground_records_usage() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("pg-usage-{:.8}@example.com", uuid::Uuid::new_v4());
    let tier_name = format!("pg-tier-{:.8}", uuid::Uuid::new_v4());
    let (user_id, _, _) = fixtures::create_user_with_subscription(&db, &email, &tier_name)
        .await
        .expect("Failed to create user");

    let token = get_auth_token(user_id, &email).await;

    let request = Request::builder()
        .uri("/api/v1/playground/text")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({ "text": "My email is test@example.com" }).to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // No sleep needed — usage recording is synchronous (M6 fix)
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM usage_logs WHERE user_id = $1 AND request_type = 'playground'",
    )
    .bind(user_id)
    .fetch_one(db.pool())
    .await
    .unwrap();

    assert!(count >= 1, "Should have at least one playground usage log");

    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

// ============================================================================
// Daily Limit Test
// ============================================================================

#[tokio::test]
async fn test_playground_daily_limit_enforced() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("pg-limit-{:.8}@example.com", uuid::Uuid::new_v4());
    let tier_name = format!("pg-tier-{:.8}", uuid::Uuid::new_v4());

    // Create user with a tier that has a daily limit of 2
    let user_id = fixtures::create_user(&db, &email, None)
        .await
        .expect("Failed to create user");

    let limits = pii_redacta_core::db::models::TierLimits {
        api_enabled: true,
        max_api_keys: Some(5),
        max_file_size: Some(10_485_760),
        max_files_per_month: Some(100),
        max_pages_per_file: Some(50),
        max_total_size: Some(524_288_000),
        playground_max_daily: Some(2),
        playground_max_file_size: Some(1_048_576),
        retention_days: Some(30),
    };

    let features = pii_redacta_core::db::models::TierFeatures {
        batch_processing: false,
        custom_rules: false,
        email_support: false,
        playground: true,
        rate_limit_per_minute: Some(60),
        sla: None,
        webhooks: false,
    };

    let tier_id = fixtures::create_tier(&db, &tier_name, "Test Limit Tier", limits, features)
        .await
        .expect("Failed to create tier");

    let _sub_id = fixtures::create_subscription(
        &db,
        user_id,
        tier_id,
        pii_redacta_core::db::models::SubscriptionStatus::Trial,
    )
    .await
    .expect("Failed to create subscription");

    let token = get_auth_token(user_id, &email).await;

    // Use up the daily limit (2 requests) — usage is recorded synchronously (C1/M6 fix)
    for i in 0..2 {
        let request = Request::builder()
            .uri("/api/v1/playground/text")
            .method("POST")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::from(
                json!({ "text": "My email is test@example.com" }).to_string(),
            ))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Request {} should succeed",
            i + 1
        );
    }

    // Third request should be rate limited
    let request = Request::builder()
        .uri("/api/v1/playground/text")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({ "text": "My email is test@example.com" }).to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "Third request should be rejected (daily limit = 2)"
    );

    // Clean up user data (subscriptions, usage_logs, etc.) and the custom tier
    fixtures::cleanup_test_data(&db, &[user_id]).await;
    let _ = sqlx::query("DELETE FROM tiers WHERE id = $1")
        .bind(tier_id)
        .execute(db.pool())
        .await;
}
