//! Authentication API Integration Tests
//!
//! These tests require PostgreSQL to be running.
//! Run with: cargo test --test auth_api_test

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

use common::setup_app;

/// Helper to parse JSON response body
async fn parse_json_response(response: axum::response::Response) -> Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    serde_json::from_slice(&body).expect("Failed to parse JSON")
}

#[tokio::test]
async fn test_register_user_success() {
    let app = setup_app().await;

    let email = format!("test-register-{:.8}@example.com", uuid::Uuid::new_v4());

    let request = Request::builder()
        .uri("/api/v1/auth/register")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "email": email,
                "password": "Test123!Pass",
                "display_name": "Test User",
                "company_name": "Test Company"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_json_response(response).await;
    assert!(body.get("token").is_some());
    assert!(body.get("user").is_some());
    assert_eq!(body["user"]["email"], email);
    assert_eq!(body["user"]["displayName"], "Test User");
    assert_eq!(body["user"]["companyName"], "Test Company");

    // Clean up - delete the created user
    let db = common::setup_db().await;
    let _ = sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(&email)
        .execute(db.pool())
        .await;
}

#[tokio::test]
async fn test_register_user_weak_password() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/api/v1/auth/register")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "email": "test@example.com",
                "password": "weak"  // Too short, no special char, etc.
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = parse_json_response(response).await;
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Password"));
}

#[tokio::test]
async fn test_register_user_invalid_email() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/api/v1/auth/register")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "email": "not-an-email",
                "password": "Test123!Pass"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = parse_json_response(response).await;
    let message = body["error"]["message"].as_str().unwrap();
    assert!(
        message.to_lowercase().contains("email"),
        "Expected error message to contain 'email', got: {}",
        message
    );
}

#[tokio::test]
async fn test_register_duplicate_email() {
    let app = setup_app().await;
    let email = format!("test-dup-{:.8}@example.com", uuid::Uuid::new_v4());

    // First registration
    let request1 = Request::builder()
        .uri("/api/v1/auth/register")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "email": &email,
                "password": "Test123!Pass"
            })
            .to_string(),
        ))
        .unwrap();

    let response1 = app.clone().oneshot(request1).await.unwrap();
    assert_eq!(response1.status(), StatusCode::OK);

    // Second registration with same email
    let request2 = Request::builder()
        .uri("/api/v1/auth/register")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "email": &email,
                "password": "Test123!Pass"
            })
            .to_string(),
        ))
        .unwrap();

    let response2 = app.oneshot(request2).await.unwrap();
    assert_eq!(response2.status(), StatusCode::CONFLICT);

    // Clean up
    let db = common::setup_db().await;
    let _ = sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(&email)
        .execute(db.pool())
        .await;
}

#[tokio::test]
async fn test_login_success() {
    let app = setup_app().await;
    let email = format!("test-login-{:.8}@example.com", uuid::Uuid::new_v4());
    let password = "Test123!Pass";

    // Register first
    let register_request = Request::builder()
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

    let register_response = app.clone().oneshot(register_request).await.unwrap();
    assert_eq!(register_response.status(), StatusCode::OK);

    // Login
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

    let login_response = app.oneshot(login_request).await.unwrap();
    assert_eq!(login_response.status(), StatusCode::OK);

    let body = parse_json_response(login_response).await;
    assert!(body.get("token").is_some());
    assert_eq!(body["user"]["email"], email);

    // Clean up
    let db = common::setup_db().await;
    let _ = sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(&email)
        .execute(db.pool())
        .await;
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/api/v1/auth/login")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "email": "nonexistent@example.com",
                "password": "WrongPassword123!"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_login_wrong_password() {
    let app = setup_app().await;
    let email = format!("test-wrong-pass-{:.8}@example.com", uuid::Uuid::new_v4());

    // Register
    let register_request = Request::builder()
        .uri("/api/v1/auth/register")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "email": &email,
                "password": "Test123!Pass"
            })
            .to_string(),
        ))
        .unwrap();

    let register_response = app.clone().oneshot(register_request).await.unwrap();
    assert_eq!(register_response.status(), StatusCode::OK);

    // Login with wrong password
    let login_request = Request::builder()
        .uri("/api/v1/auth/login")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "email": &email,
                "password": "WrongPassword123!"
            })
            .to_string(),
        ))
        .unwrap();

    let login_response = app.oneshot(login_request).await.unwrap();
    assert_eq!(login_response.status(), StatusCode::UNAUTHORIZED);

    // Clean up
    let db = common::setup_db().await;
    let _ = sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(&email)
        .execute(db.pool())
        .await;
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/health")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_json_response(response).await;
    assert_eq!(body["status"], "healthy");
    assert!(body.get("version").is_some());
}

#[tokio::test]
async fn test_logout_endpoint() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/api/v1/auth/logout")
        .method("POST")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Logout returns 204 No Content in stateless JWT system
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_cors_preflight() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/api/v1/auth/login")
        .method("OPTIONS")
        .header("Origin", "http://localhost:3000")
        .header("Access-Control-Request-Method", "POST")
        .header("Access-Control-Request-Headers", "Content-Type")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response
        .headers()
        .get("access-control-allow-origin")
        .is_some());
    assert!(response
        .headers()
        .get("access-control-allow-methods")
        .is_some());
}
