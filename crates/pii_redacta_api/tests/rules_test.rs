//! Integration tests for Custom Rules endpoints
//!
//! Sprint 14: Custom Rules CRUD + test
//!
//! Run with: cargo test --test rules_test

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

/// Helper to create a user with custom_rules enabled
async fn create_pro_user(
    db: &pii_redacta_core::db::Database,
    email: &str,
    tier_name: &str,
) -> (uuid::Uuid, uuid::Uuid) {
    let user_id = fixtures::create_user(db, email, None)
        .await
        .expect("Failed to create user");

    let limits = pii_redacta_core::db::models::TierLimits {
        api_enabled: true,
        max_custom_rules: Some(10),
        ..Default::default()
    };

    let features = pii_redacta_core::db::models::TierFeatures {
        custom_rules: true,
        playground: true,
        ..Default::default()
    };

    let tier_id = fixtures::create_tier(db, tier_name, "Pro Test", limits, features)
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

// ============================================================================
// Auth Tests
// ============================================================================

#[tokio::test]
async fn test_create_rule_requires_auth() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/api/v1/rules")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "name": "Test",
                "pattern": r"\d+",
                "entityLabel": "NUMBER"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_rule_requires_feature() {
    let app = setup_app().await;
    let db = setup_db().await;

    // Create user with trial tier (custom_rules = false)
    let email = format!("rt-{}@test.com", short_id());
    let tier_name = format!("rt-t-{}", short_id());
    let (user_id, _, _) = fixtures::create_user_with_subscription(&db, &email, &tier_name)
        .await
        .expect("Failed to create user");

    let token = get_auth_token(user_id, &email).await;

    let request = Request::builder()
        .uri("/api/v1/rules")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "name": "Test",
                "pattern": r"\d+",
                "entityLabel": "NUMBER"
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
async fn test_create_rule_success() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("rc-{}@test.com", short_id());
    let tier_name = format!("rp-t-{}", short_id());
    let (user_id, _) = create_pro_user(&db, &email, &tier_name).await;
    let token = get_auth_token(user_id, &email).await;

    let request = Request::builder()
        .uri("/api/v1/rules")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "name": "Employee ID",
                "description": "Detects employee IDs",
                "pattern": r"EMP-\d{6}",
                "entityLabel": "EMPLOYEE_ID",
                "confidence": 0.95
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = parse_json(response).await;
    assert_eq!(body["name"], "Employee ID");
    assert_eq!(body["entityLabel"], "EMPLOYEE_ID");
    assert!(body["id"].is_string());
    assert!(body["createdAt"].is_string());

    // Cleanup
    let _ = sqlx::query("DELETE FROM custom_rules WHERE user_id = $1")
        .bind(user_id)
        .execute(db.pool())
        .await;
    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

#[tokio::test]
async fn test_list_rules() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("rl-{}@test.com", short_id());
    let tier_name = format!("rl-t-{}", short_id());
    let (user_id, _) = create_pro_user(&db, &email, &tier_name).await;
    let token = get_auth_token(user_id, &email).await;

    // Create 2 rules
    for i in 0..2 {
        let req = Request::builder()
            .uri("/api/v1/rules")
            .method("POST")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::from(
                json!({
                    "name": format!("Rule {}", i),
                    "pattern": format!(r"PAT{}\d+", i),
                    "entityLabel": format!("TYPE_{}", i),
                })
                .to_string(),
            ))
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    // List
    let list_req = Request::builder()
        .uri("/api/v1/rules")
        .method("GET")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(list_req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_json(response).await;
    let rules = body.as_array().unwrap();
    assert_eq!(rules.len(), 2);

    let _ = sqlx::query("DELETE FROM custom_rules WHERE user_id = $1")
        .bind(user_id)
        .execute(db.pool())
        .await;
    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

#[tokio::test]
async fn test_update_rule() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("ru-{}@test.com", short_id());
    let tier_name = format!("ru-t-{}", short_id());
    let (user_id, _) = create_pro_user(&db, &email, &tier_name).await;
    let token = get_auth_token(user_id, &email).await;

    // Create
    let create_req = Request::builder()
        .uri("/api/v1/rules")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "name": "Original Name",
                "pattern": r"\d{4}",
                "entityLabel": "CODE",
            })
            .to_string(),
        ))
        .unwrap();

    let resp = app.clone().oneshot(create_req).await.unwrap();
    let body = parse_json(resp).await;
    let rule_id = body["id"].as_str().unwrap().to_string();

    // Update
    let update_req = Request::builder()
        .uri(format!("/api/v1/rules/{}", rule_id))
        .method("PUT")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "name": "Updated Name",
                "pattern": r"\d{6}",
            })
            .to_string(),
        ))
        .unwrap();

    let update_resp = app.oneshot(update_req).await.unwrap();
    assert_eq!(update_resp.status(), StatusCode::OK);

    let updated = parse_json(update_resp).await;
    assert_eq!(updated["name"], "Updated Name");
    assert_eq!(updated["pattern"], r"\d{6}");

    let _ = sqlx::query("DELETE FROM custom_rules WHERE user_id = $1")
        .bind(user_id)
        .execute(db.pool())
        .await;
    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

#[tokio::test]
async fn test_delete_rule() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("rd-{}@test.com", short_id());
    let tier_name = format!("rd-t-{}", short_id());
    let (user_id, _) = create_pro_user(&db, &email, &tier_name).await;
    let token = get_auth_token(user_id, &email).await;

    // Create
    let create_req = Request::builder()
        .uri("/api/v1/rules")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "name": "To Delete",
                "pattern": r"\d+",
                "entityLabel": "DEL",
            })
            .to_string(),
        ))
        .unwrap();

    let resp = app.clone().oneshot(create_req).await.unwrap();
    let body = parse_json(resp).await;
    let rule_id = body["id"].as_str().unwrap().to_string();

    // Delete
    let delete_req = Request::builder()
        .uri(format!("/api/v1/rules/{}", rule_id))
        .method("DELETE")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let del_resp = app.clone().oneshot(delete_req).await.unwrap();
    assert_eq!(del_resp.status(), StatusCode::NO_CONTENT);

    // Verify 404 on GET
    let get_req = Request::builder()
        .uri(format!("/api/v1/rules/{}", rule_id))
        .method("GET")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let get_resp = app.oneshot(get_req).await.unwrap();
    assert_eq!(get_resp.status(), StatusCode::NOT_FOUND);

    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

#[tokio::test]
async fn test_test_rule() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("rte-{}@test.com", short_id());
    let tier_name = format!("rtt-{}", short_id());
    let (user_id, _) = create_pro_user(&db, &email, &tier_name).await;
    let token = get_auth_token(user_id, &email).await;

    // Create a rule
    let create_req = Request::builder()
        .uri("/api/v1/rules")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "name": "Phone Pattern",
                "pattern": r"\+60\d{9,10}",
                "entityLabel": "MY_PHONE",
            })
            .to_string(),
        ))
        .unwrap();

    let resp = app.clone().oneshot(create_req).await.unwrap();
    let body = parse_json(resp).await;
    let rule_id = body["id"].as_str().unwrap().to_string();

    // Test the rule
    let test_req = Request::builder()
        .uri(format!("/api/v1/rules/{}/test", rule_id))
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({ "text": "Call me at +60123456789 or +60198765432" }).to_string(),
        ))
        .unwrap();

    let test_resp = app.oneshot(test_req).await.unwrap();
    assert_eq!(test_resp.status(), StatusCode::OK);

    let result = parse_json(test_resp).await;
    let matches = result["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0]["entityLabel"], "MY_PHONE");

    let _ = sqlx::query("DELETE FROM custom_rules WHERE user_id = $1")
        .bind(user_id)
        .execute(db.pool())
        .await;
    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

#[tokio::test]
async fn test_invalid_pattern_rejected() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("rb-{}@test.com", short_id());
    let tier_name = format!("rb-t-{}", short_id());
    let (user_id, _) = create_pro_user(&db, &email, &tier_name).await;
    let token = get_auth_token(user_id, &email).await;

    let request = Request::builder()
        .uri("/api/v1/rules")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "name": "Bad Pattern",
                "pattern": r"(unclosed",
                "entityLabel": "BAD",
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    fixtures::cleanup_test_data(&db, &[user_id]).await;
}
