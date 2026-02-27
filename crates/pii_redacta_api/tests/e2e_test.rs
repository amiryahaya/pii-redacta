//! End-to-End User Flow Tests
//!
//! These tests simulate complete user workflows.
//! Run with: cargo test --test e2e_test

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

use common::{setup_app, setup_db};

/// Helper to parse JSON response body
async fn parse_json_response(response: axum::response::Response) -> Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    serde_json::from_slice(&body).expect("Failed to parse JSON")
}

/// E2E Test: Complete user registration and login flow
#[tokio::test]
async fn test_user_registration_login_flow() {
    let app = setup_app().await;
    let db = setup_db().await;
    let email = format!("e2e-user-{:.8}@example.com", uuid::Uuid::new_v4());
    let password = "SecurePass123!";

    // Step 1: Register
    let register_request = Request::builder()
        .uri("/api/v1/auth/register")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "email": &email,
                "password": password,
                "display_name": "E2E Test User",
                "company_name": "E2E Test Company"
            })
            .to_string(),
        ))
        .unwrap();

    let register_response = app.clone().oneshot(register_request).await.unwrap();
    assert_eq!(register_response.status(), StatusCode::OK);

    let register_body = parse_json_response(register_response).await;
    assert!(register_body.get("token").is_some());
    assert!(register_body.get("user").is_some());
    let user_id = register_body["user"]["id"].as_str().unwrap();

    // Step 2: Login with same credentials
    let login_request = Request::builder()
        .uri("/api/v1/auth/login")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "email": &email,
                "password": password
            })
            .to_string(),
        ))
        .unwrap();

    let login_response = app.clone().oneshot(login_request).await.unwrap();
    assert_eq!(login_response.status(), StatusCode::OK);

    let login_body = parse_json_response(login_response).await;
    assert!(login_body.get("token").is_some());
    assert_eq!(login_body["user"]["email"], email);
    assert_eq!(login_body["user"]["id"], user_id);

    // Step 3: Check health endpoint is accessible
    let health_request = Request::builder()
        .uri("/health")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let health_response = app.oneshot(health_request).await.unwrap();
    assert_eq!(health_response.status(), StatusCode::OK);

    // Clean up
    let _ = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(uuid::Uuid::parse_str(user_id).unwrap())
        .execute(db.pool())
        .await;
}

/// E2E Test: User tries to register with existing email
#[tokio::test]
async fn test_duplicate_registration_attempt() {
    let app = setup_app().await;
    let db = setup_db().await;
    let email = format!("e2e-dup-{:.8}@example.com", uuid::Uuid::new_v4());

    // First registration
    let request1 = Request::builder()
        .uri("/api/v1/auth/register")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "email": &email,
                "password": "SecurePass123!"
            })
            .to_string(),
        ))
        .unwrap();

    let response1 = app.clone().oneshot(request1).await.unwrap();
    assert_eq!(response1.status(), StatusCode::OK);

    // Second registration with same email should fail
    let request2 = Request::builder()
        .uri("/api/v1/auth/register")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "email": &email,
                "password": "DifferentPass123!"
            })
            .to_string(),
        ))
        .unwrap();

    let response2 = app.clone().oneshot(request2).await.unwrap();
    assert_eq!(response2.status(), StatusCode::CONFLICT);

    let body = parse_json_response(response2).await;
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("already exists"));

    // Clean up
    let _ = sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(&email)
        .execute(db.pool())
        .await;
}

/// E2E Test: Various password validation scenarios
#[tokio::test]
async fn test_password_validation_scenarios() {
    let app = setup_app().await;
    let email_prefix = format!("e2e-pass-{:.8}", uuid::Uuid::new_v4());

    let test_cases = vec![
        ("short", false, "too short"),
        ("nouppercase123!", false, "no uppercase"),
        ("NOLOWERCASE123!", false, "no lowercase"),
        ("NoNumbers!", false, "no numbers"),
        ("NoSpecial123", false, "no special char"),
        ("ValidPass123!", true, "valid password"),
    ];

    for (i, (password, should_succeed, description)) in test_cases.iter().enumerate() {
        let email = format!("{}-{}@example.com", email_prefix, i);

        let request = Request::builder()
            .uri("/api/v1/auth/register")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(
                json!({
                    "email": &email,
                    "password": password
                })
                .to_string(),
            ))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        if *should_succeed {
            assert_eq!(
                response.status(),
                StatusCode::OK,
                "Password '{}' should be valid ({})",
                password,
                description
            );
        } else {
            assert_eq!(
                response.status(),
                StatusCode::BAD_REQUEST,
                "Password '{}' should be invalid ({})",
                password,
                description
            );
        }

        // Clean up if successful
        if *should_succeed {
            let _ = sqlx::query("DELETE FROM users WHERE email = $1")
                .bind(&email)
                .execute(setup_db().await.pool())
                .await;
        }
    }
}

/// E2E Test: CORS headers in responses
#[tokio::test]
async fn test_cors_headers_present() {
    let app = setup_app().await;

    let origins = vec![
        "http://localhost:3000",
        "http://localhost:5173",
        "http://127.0.0.1:3000",
    ];

    for origin in origins {
        let request = Request::builder()
            .uri("/health")
            .method("GET")
            .header("Origin", origin)
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let cors_header = response.headers().get("access-control-allow-origin");
        assert!(
            cors_header.is_some(),
            "CORS header missing for origin {}",
            origin
        );
    }
}

/// E2E Test: Security headers in responses
#[tokio::test]
async fn test_security_headers_present() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/health")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Check security headers
    assert!(response.headers().get("x-content-type-options").is_some());
    assert!(response.headers().get("x-frame-options").is_some());
    assert!(response.headers().get("x-xss-protection").is_some());
    assert!(response.headers().get("referrer-policy").is_some());
    assert!(response.headers().get("content-security-policy").is_some());
}

/// E2E Test: Concurrent user registrations
#[tokio::test]
async fn test_concurrent_registrations() {
    let email_prefix = format!("concurrent-{:.8}", uuid::Uuid::new_v4());

    let mut handles = vec![];

    for i in 0..5 {
        let email = format!("{}-{}@example.com", email_prefix, i);
        let handle = tokio::spawn(async move {
            let app = setup_app().await;

            let request = Request::builder()
                .uri("/api/v1/auth/register")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "email": &email,
                        "password": "SecurePass123!"
                    })
                    .to_string(),
                ))
                .unwrap();

            let response = app.oneshot(request).await.unwrap();
            (email, response.status())
        });
        handles.push(handle);
    }

    let results: Vec<(String, StatusCode)> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    // All registrations should succeed (different emails)
    for (email, status) in &results {
        assert_eq!(*status, StatusCode::OK, "Registration failed for {}", email);
    }

    // Clean up
    let db = setup_db().await;
    for (email, _) in results {
        let _ = sqlx::query("DELETE FROM users WHERE email = $1")
            .bind(&email)
            .execute(db.pool())
            .await;
    }
}

/// E2E Test: Database health check
#[tokio::test]
async fn test_database_connectivity() {
    let db = setup_db().await;

    // Test basic connectivity
    let result: Result<(i32,), sqlx::Error> = sqlx::query_as("SELECT 1").fetch_one(db.pool()).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().0, 1);

    // Test that migrations have run (check for expected tables)
    let tables: Vec<(String,)> = sqlx::query_as(
        "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'",
    )
    .fetch_all(db.pool())
    .await
    .expect("Failed to query tables");

    let table_names: Vec<String> = tables.into_iter().map(|t| t.0).collect();
    assert!(table_names.contains(&"users".to_string()));
    assert!(table_names.contains(&"tiers".to_string()));
    assert!(table_names.contains(&"subscriptions".to_string()));
    assert!(table_names.contains(&"api_keys".to_string()));
}
