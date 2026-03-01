//! Integration tests for Webhook endpoints
//!
//! Sprint 14: Webhook CRUD, SSRF protection, delivery logs
//!
//! Run with: cargo test --test webhooks_test

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

/// Helper to create a user with webhooks enabled
async fn create_webhook_user(
    db: &pii_redacta_core::db::Database,
    email: &str,
    tier_name: &str,
    max_webhook_endpoints: Option<i32>,
) -> (uuid::Uuid, uuid::Uuid) {
    let user_id = fixtures::create_user(db, email, None)
        .await
        .expect("Failed to create user");

    let limits = pii_redacta_core::db::models::TierLimits {
        api_enabled: true,
        max_webhook_endpoints,
        ..Default::default()
    };

    let features = pii_redacta_core::db::models::TierFeatures {
        webhooks: true,
        playground: true,
        ..Default::default()
    };

    let tier_id = fixtures::create_tier(db, tier_name, "Webhook Test", limits, features)
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

async fn cleanup_webhook_data(db: &pii_redacta_core::db::Database, user_id: uuid::Uuid) {
    // Deliveries cascade from endpoints
    let _ = sqlx::query("DELETE FROM webhook_endpoints WHERE user_id = $1")
        .bind(user_id)
        .execute(db.pool())
        .await;
    fixtures::cleanup_test_data(db, &[user_id]).await;
}

// ============================================================================
// Auth Tests
// ============================================================================

#[tokio::test]
async fn test_webhook_requires_auth() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/api/v1/webhooks")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "url": "https://example.com/webhook",
                "events": ["detection.completed"]
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_webhook_requires_feature() {
    let app = setup_app().await;
    let db = setup_db().await;

    // Default fixture has webhooks = false
    let email = format!("wt-{}@test.com", short_id());
    let tier_name = format!("wt-t-{}", short_id());
    let (user_id, _, _) = fixtures::create_user_with_subscription(&db, &email, &tier_name)
        .await
        .expect("Failed to create user");

    let token = get_auth_token(user_id, &email).await;

    let request = Request::builder()
        .uri("/api/v1/webhooks")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "url": "https://example.com/webhook",
                "events": ["detection.completed"]
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

// ============================================================================
// CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_webhook_success() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("wc-{}@test.com", short_id());
    let tier_name = format!("wc-t-{}", short_id());
    let (user_id, _) = create_webhook_user(&db, &email, &tier_name, Some(5)).await;
    let token = get_auth_token(user_id, &email).await;

    let request = Request::builder()
        .uri("/api/v1/webhooks")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "url": "https://example.com/webhook",
                "description": "Test webhook",
                "events": ["detection.completed", "batch.completed"]
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_json(response).await;
    assert!(body["id"].is_string());
    assert_eq!(body["url"], "https://example.com/webhook");
    assert!(body["secret"].is_string());
    // Secret should be full on creation
    let secret = body["secret"].as_str().unwrap();
    assert!(secret.len() > 10, "Secret should be full on creation");
    assert_eq!(body["isActive"], true);

    cleanup_webhook_data(&db, user_id).await;
}

#[tokio::test]
async fn test_create_webhook_invalid_url() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("wb-{}@test.com", short_id());
    let tier_name = format!("wb-t-{}", short_id());
    let (user_id, _) = create_webhook_user(&db, &email, &tier_name, Some(5)).await;
    let token = get_auth_token(user_id, &email).await;

    // HTTP (not HTTPS)
    let request = Request::builder()
        .uri("/api/v1/webhooks")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "url": "http://example.com/webhook",
                "events": ["detection.completed"]
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    cleanup_webhook_data(&db, user_id).await;
}

#[tokio::test]
async fn test_create_webhook_ssrf_blocked() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("ws-{}@test.com", short_id());
    let tier_name = format!("ws-t-{}", short_id());
    let (user_id, _) = create_webhook_user(&db, &email, &tier_name, Some(5)).await;
    let token = get_auth_token(user_id, &email).await;

    let private_urls = vec![
        "https://localhost/webhook",
        "https://127.0.0.1/webhook",
        "https://10.0.0.1/webhook",
        "https://192.168.1.1/webhook",
    ];

    for url in private_urls {
        let request = Request::builder()
            .uri("/api/v1/webhooks")
            .method("POST")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::from(
                json!({
                    "url": url,
                    "events": ["detection.completed"]
                })
                .to_string(),
            ))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Should reject private URL: {}",
            url
        );
    }

    cleanup_webhook_data(&db, user_id).await;
}

#[tokio::test]
async fn test_list_webhooks() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("wl-{}@test.com", short_id());
    let tier_name = format!("wl-t-{}", short_id());
    let (user_id, _) = create_webhook_user(&db, &email, &tier_name, Some(5)).await;
    let token = get_auth_token(user_id, &email).await;

    // Create 2 webhooks
    for i in 0..2 {
        let req = Request::builder()
            .uri("/api/v1/webhooks")
            .method("POST")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::from(
                json!({
                    "url": format!("https://example{}.com/webhook", i),
                    "events": ["detection.completed"]
                })
                .to_string(),
            ))
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    // List
    let list_req = Request::builder()
        .uri("/api/v1/webhooks")
        .method("GET")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(list_req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_json(response).await;
    let webhooks = body.as_array().unwrap();
    assert_eq!(webhooks.len(), 2);

    // Secret should be masked in list
    for wh in webhooks {
        let secret = wh["secret"].as_str().unwrap();
        assert!(secret.ends_with("..."), "Secret should be masked in list");
    }

    cleanup_webhook_data(&db, user_id).await;
}

#[tokio::test]
async fn test_delete_webhook() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("wd-{}@test.com", short_id());
    let tier_name = format!("wd-t-{}", short_id());
    let (user_id, _) = create_webhook_user(&db, &email, &tier_name, Some(5)).await;
    let token = get_auth_token(user_id, &email).await;

    // Create
    let create_req = Request::builder()
        .uri("/api/v1/webhooks")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "url": "https://example.com/webhook",
                "events": ["detection.completed"]
            })
            .to_string(),
        ))
        .unwrap();

    let resp = app.clone().oneshot(create_req).await.unwrap();
    let body = parse_json(resp).await;
    let wh_id = body["id"].as_str().unwrap().to_string();

    // Delete
    let delete_req = Request::builder()
        .uri(format!("/api/v1/webhooks/{}", wh_id))
        .method("DELETE")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let del_resp = app.clone().oneshot(delete_req).await.unwrap();
    assert_eq!(del_resp.status(), StatusCode::NO_CONTENT);

    // Verify 404
    let get_req = Request::builder()
        .uri(format!("/api/v1/webhooks/{}", wh_id))
        .method("GET")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let get_resp = app.oneshot(get_req).await.unwrap();
    assert_eq!(get_resp.status(), StatusCode::NOT_FOUND);

    cleanup_webhook_data(&db, user_id).await;
}

#[tokio::test]
async fn test_webhook_deliveries_list() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("wdv-{}@test.com", short_id());
    let tier_name = format!("wdv-t-{}", short_id());
    let (user_id, _) = create_webhook_user(&db, &email, &tier_name, Some(5)).await;
    let token = get_auth_token(user_id, &email).await;

    // Create webhook
    let create_req = Request::builder()
        .uri("/api/v1/webhooks")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "url": "https://example.com/webhook",
                "events": ["detection.completed"]
            })
            .to_string(),
        ))
        .unwrap();

    let resp = app.clone().oneshot(create_req).await.unwrap();
    let body = parse_json(resp).await;
    let wh_id = body["id"].as_str().unwrap().to_string();

    // List deliveries (should be empty initially)
    let list_req = Request::builder()
        .uri(format!("/api/v1/webhooks/{}/deliveries", wh_id))
        .method("GET")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let list_resp = app.oneshot(list_req).await.unwrap();
    assert_eq!(list_resp.status(), StatusCode::OK);

    let deliveries = parse_json(list_resp).await;
    assert!(deliveries.is_array());

    cleanup_webhook_data(&db, user_id).await;
}

#[tokio::test]
async fn test_webhook_limit_enforced() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("wlm-{}@test.com", short_id());
    let tier_name = format!("wlm-t-{}", short_id());
    // Max 1 webhook
    let (user_id, _) = create_webhook_user(&db, &email, &tier_name, Some(1)).await;
    let token = get_auth_token(user_id, &email).await;

    // Create first webhook (should succeed)
    let req1 = Request::builder()
        .uri("/api/v1/webhooks")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "url": "https://example.com/webhook1",
                "events": ["detection.completed"]
            })
            .to_string(),
        ))
        .unwrap();

    let resp1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::CREATED);

    // Create second webhook (should fail)
    let req2 = Request::builder()
        .uri("/api/v1/webhooks")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "url": "https://example.com/webhook2",
                "events": ["detection.completed"]
            })
            .to_string(),
        ))
        .unwrap();

    let resp2 = app.oneshot(req2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::TOO_MANY_REQUESTS);

    cleanup_webhook_data(&db, user_id).await;
}
