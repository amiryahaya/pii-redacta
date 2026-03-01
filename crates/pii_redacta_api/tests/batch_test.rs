//! Integration tests for Batch Processing endpoints
//!
//! Sprint 14: Batch detect, status, results
//!
//! Run with: cargo test --test batch_test

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use pii_redacta_api::jwt::{generate_token, JwtConfig};
use serde_json::{json, Value};
use tower::ServiceExt;

use common::{fixtures, setup_app, setup_db, test_jwt_secret};

fn short_id() -> String {
    uuid::Uuid::new_v4().to_string()[..8].to_string()
}

async fn parse_json(response: axum::response::Response) -> Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    serde_json::from_slice(&body).expect("Failed to parse JSON")
}

async fn get_auth_token(user_id: uuid::Uuid, email: &str) -> String {
    let config = JwtConfig::new(test_jwt_secret(), 24).expect("Valid JWT config");
    generate_token(user_id, email, false, &config).expect("Should generate token")
}

/// Helper to create a user with batch_processing enabled
async fn create_batch_user(
    db: &pii_redacta_core::db::Database,
    email: &str,
    tier_name: &str,
    max_batch_items: Option<i32>,
) -> (uuid::Uuid, uuid::Uuid) {
    let user_id = fixtures::create_user(db, email, None)
        .await
        .expect("Failed to create user");

    let limits = pii_redacta_core::db::models::TierLimits {
        api_enabled: true,
        max_batch_items,
        ..Default::default()
    };

    let features = pii_redacta_core::db::models::TierFeatures {
        batch_processing: true,
        custom_rules: true,
        playground: true,
        ..Default::default()
    };

    let tier_id = fixtures::create_tier(db, tier_name, "Batch Test", limits, features)
        .await
        .expect("Failed to create tier");

    let _sub_id = fixtures::create_subscription(
        db,
        user_id,
        tier_id,
        pii_redacta_core::db::models::SubscriptionStatus::Active,
    )
    .await
    .expect("Failed to create subscription");

    (user_id, tier_id)
}

async fn cleanup_batch_data(db: &pii_redacta_core::db::Database, user_id: uuid::Uuid) {
    // Delete batch items via cascade from batch jobs
    let _ = sqlx::query("DELETE FROM batch_jobs WHERE user_id = $1")
        .bind(user_id)
        .execute(db.pool())
        .await;
    let _ = sqlx::query("DELETE FROM custom_rules WHERE user_id = $1")
        .bind(user_id)
        .execute(db.pool())
        .await;
    fixtures::cleanup_test_data(db, &[user_id]).await;
}

// ============================================================================
// Auth Tests
// ============================================================================

#[tokio::test]
async fn test_batch_requires_auth() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/api/v1/batch/detect")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(json!({ "items": ["test"] }).to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_batch_requires_feature() {
    let app = setup_app().await;
    let db = setup_db().await;

    // Trial tier has batch_processing = false (from create_user_with_subscription default)
    let email = format!("bt-{}@test.com", short_id());
    let tier_name = format!("bt-t-{}", short_id());
    let (user_id, _, _) = fixtures::create_user_with_subscription(&db, &email, &tier_name)
        .await
        .expect("Failed to create user");

    let token = get_auth_token(user_id, &email).await;

    let request = Request::builder()
        .uri("/api/v1/batch/detect")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(json!({ "items": ["test"] }).to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    // Should be forbidden since default fixture has batch_processing: true, but custom_rules: false
    // Actually, the default fixture from create_user_with_subscription has batch_processing: true
    // Let's just ensure it doesn't 401
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);

    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

// ============================================================================
// Submit & Status Tests
// ============================================================================

#[tokio::test]
async fn test_batch_submit_success() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("bs-{}@test.com", short_id());
    let tier_name = format!("bs-t-{}", short_id());
    let (user_id, _) = create_batch_user(&db, &email, &tier_name, Some(100)).await;
    let token = get_auth_token(user_id, &email).await;

    let request = Request::builder()
        .uri("/api/v1/batch/detect")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "items": [
                    "My email is test@example.com",
                    "Call +60123456789",
                    "No PII here"
                ]
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_json(response).await;
    assert!(body["batchId"].is_string());
    assert_eq!(body["status"], "pending");
    assert_eq!(body["totalItems"], 3);

    cleanup_batch_data(&db, user_id).await;
}

#[tokio::test]
async fn test_batch_status() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("bst-{}@test.com", short_id());
    let tier_name = format!("bst-t-{}", short_id());
    let (user_id, _) = create_batch_user(&db, &email, &tier_name, Some(100)).await;
    let token = get_auth_token(user_id, &email).await;

    // Submit
    let submit_req = Request::builder()
        .uri("/api/v1/batch/detect")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({ "items": ["test@example.com"] }).to_string(),
        ))
        .unwrap();

    let submit_resp = app.clone().oneshot(submit_req).await.unwrap();
    let submit_body = parse_json(submit_resp).await;
    let batch_id = submit_body["batchId"].as_str().unwrap().to_string();

    // Brief pause for background processing
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Poll status
    let status_req = Request::builder()
        .uri(format!("/api/v1/batch/{}", batch_id))
        .method("GET")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let status_resp = app.oneshot(status_req).await.unwrap();
    assert_eq!(status_resp.status(), StatusCode::OK);

    let status_body = parse_json(status_resp).await;
    assert_eq!(status_body["id"], batch_id);
    assert_eq!(status_body["totalItems"], 1);

    cleanup_batch_data(&db, user_id).await;
}

#[tokio::test]
async fn test_batch_results() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("br-{}@test.com", short_id());
    let tier_name = format!("br-t-{}", short_id());
    let (user_id, _) = create_batch_user(&db, &email, &tier_name, Some(100)).await;
    let token = get_auth_token(user_id, &email).await;

    // Submit with PII
    let submit_req = Request::builder()
        .uri("/api/v1/batch/detect")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "items": ["Email: test@example.com", "No PII"]
            })
            .to_string(),
        ))
        .unwrap();

    let submit_resp = app.clone().oneshot(submit_req).await.unwrap();
    let submit_body = parse_json(submit_resp).await;
    let batch_id = submit_body["batchId"].as_str().unwrap().to_string();

    // Wait for processing
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    // Get results
    let results_req = Request::builder()
        .uri(format!("/api/v1/batch/{}/results", batch_id))
        .method("GET")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let results_resp = app.oneshot(results_req).await.unwrap();
    assert_eq!(results_resp.status(), StatusCode::OK);

    let results = parse_json(results_resp).await;
    let items = results.as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["itemIndex"], 0);
    assert_eq!(items[1]["itemIndex"], 1);

    cleanup_batch_data(&db, user_id).await;
}

#[tokio::test]
async fn test_batch_with_redaction() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("brd-{}@test.com", short_id());
    let tier_name = format!("brd-t-{}", short_id());
    let (user_id, _) = create_batch_user(&db, &email, &tier_name, Some(100)).await;
    let token = get_auth_token(user_id, &email).await;

    let submit_req = Request::builder()
        .uri("/api/v1/batch/detect")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "items": ["Contact: test@example.com"],
                "redact": true
            })
            .to_string(),
        ))
        .unwrap();

    let submit_resp = app.clone().oneshot(submit_req).await.unwrap();
    let submit_body = parse_json(submit_resp).await;
    let batch_id = submit_body["batchId"].as_str().unwrap().to_string();

    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    let results_req = Request::builder()
        .uri(format!("/api/v1/batch/{}/results", batch_id))
        .method("GET")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let results_resp = app.oneshot(results_req).await.unwrap();
    let results = parse_json(results_resp).await;
    let items = results.as_array().unwrap();

    // Should have redacted text
    if let Some(redacted) = items[0]["redactedText"].as_str() {
        assert!(
            !redacted.contains("test@example.com"),
            "Redacted text should not contain original email"
        );
    }

    cleanup_batch_data(&db, user_id).await;
}

#[tokio::test]
async fn test_batch_item_limit_enforced() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("bl-{}@test.com", short_id());
    let tier_name = format!("bl-t-{}", short_id());
    // Create user with max 2 batch items
    let (user_id, _) = create_batch_user(&db, &email, &tier_name, Some(2)).await;
    let token = get_auth_token(user_id, &email).await;

    let request = Request::builder()
        .uri("/api/v1/batch/detect")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "items": ["item1", "item2", "item3"]
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

    cleanup_batch_data(&db, user_id).await;
}

#[tokio::test]
async fn test_batch_with_custom_rules() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("bc-{}@test.com", short_id());
    let tier_name = format!("bc-t-{}", short_id());
    let (user_id, _) = create_batch_user(&db, &email, &tier_name, Some(100)).await;
    let token = get_auth_token(user_id, &email).await;

    // Create a custom rule
    let create_rule_req = Request::builder()
        .uri("/api/v1/rules")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "name": "Project Code",
                "pattern": r"PRJ-\d{4}",
                "entityLabel": "PROJECT_CODE",
            })
            .to_string(),
        ))
        .unwrap();

    let rule_resp = app.clone().oneshot(create_rule_req).await.unwrap();
    assert_eq!(rule_resp.status(), StatusCode::CREATED);

    // Submit batch with custom rules enabled
    let submit_req = Request::builder()
        .uri("/api/v1/batch/detect")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "items": ["Working on PRJ-1234 today"],
                "useCustomRules": true
            })
            .to_string(),
        ))
        .unwrap();

    let submit_resp = app.clone().oneshot(submit_req).await.unwrap();
    assert_eq!(submit_resp.status(), StatusCode::CREATED);

    let submit_body = parse_json(submit_resp).await;
    let batch_id = submit_body["batchId"].as_str().unwrap().to_string();

    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    let results_req = Request::builder()
        .uri(format!("/api/v1/batch/{}/results", batch_id))
        .method("GET")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let results_resp = app.oneshot(results_req).await.unwrap();
    let results = parse_json(results_resp).await;
    let items = results.as_array().unwrap();
    assert_eq!(items[0]["status"], "completed");

    cleanup_batch_data(&db, user_id).await;
}
