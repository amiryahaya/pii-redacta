//! Authentication & User Profile API Integration Tests
//!
//! Covers: registration, login, logout, /auth/me, /users/me, preferences,
//! change-password, health, and token rejection (invalid/expired).
//!
//! Run with: cargo test --test auth_api_test

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

/// Helper to generate an expired token
fn get_expired_token(user_id: uuid::Uuid, email: &str) -> String {
    use jsonwebtoken::{encode, EncodingKey, Header};
    use serde::Serialize;

    #[derive(Serialize)]
    struct Claims {
        sub: String,
        iat: i64,
        exp: i64,
        email: String,
        is_admin: bool,
    }

    let now = chrono::Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        iat: (now - chrono::Duration::hours(2)).timestamp(),
        exp: (now - chrono::Duration::hours(1)).timestamp(),
        email: email.to_string(),
        is_admin: false,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(test_jwt_secret().as_bytes()),
    )
    .expect("Should encode token")
}

// ============================================================================
// Registration Tests
// ============================================================================

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
                "displayName": "Test User",
                "companyName": "Test Company"
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

    // Clean up
    let db = setup_db().await;
    let _ = sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(&email)
        .execute(db.pool())
        .await;
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
    let db = setup_db().await;
    let _ = sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(&email)
        .execute(db.pool())
        .await;
}

/// Password validation — replaces single weak-password test with 6 cases
#[tokio::test]
async fn test_password_validation_scenarios() {
    let app = setup_app().await;
    let email_prefix = format!("auth-pass-{:.8}", uuid::Uuid::new_v4());

    let test_cases = [
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
            let db = setup_db().await;
            let _ = sqlx::query("DELETE FROM users WHERE email = $1")
                .bind(&email)
                .execute(db.pool())
                .await;
        }
    }
}

// ============================================================================
// Login Tests
// ============================================================================

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
    let db = setup_db().await;
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
    let db = setup_db().await;
    let _ = sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(&email)
        .execute(db.pool())
        .await;
}

// ============================================================================
// Health & Logout Tests
// ============================================================================

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
async fn test_health_deep_endpoint() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/health/deep")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_json_response(response).await;
    assert_eq!(body["status"], "healthy");
    assert!(body.get("version").is_some());
    assert!(
        body.get("dependencies").is_some(),
        "Deep health should include dependencies"
    );
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

// ============================================================================
// 401 Rejection Tests — No Token
// ============================================================================

#[tokio::test]
async fn test_get_me_requires_auth() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/api/v1/auth/me")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_change_password_requires_auth() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/api/v1/auth/change-password")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "currentPassword": "OldPass123!",
                "newPassword": "NewPass123!"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_users_me_requires_auth() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/api/v1/users/me")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_put_users_me_requires_auth() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/api/v1/users/me")
        .method("PUT")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "displayName": "Hacker"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_preferences_requires_auth() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/api/v1/users/me/preferences")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_put_preferences_requires_auth() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/api/v1/users/me/preferences")
        .method("PUT")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "emailQuotaAlert": true,
                "emailSecurityAlert": true,
                "emailMarketing": false,
                "emailMonthlyReport": true
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ============================================================================
// 401 Rejection Tests — Invalid / Expired Token
// ============================================================================

#[tokio::test]
async fn test_get_me_rejects_invalid_token() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/api/v1/auth/me")
        .method("GET")
        .header("Authorization", "Bearer invalid.token.here")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_me_rejects_expired_token() {
    let app = setup_app().await;
    let user_id = uuid::Uuid::new_v4();
    let token = get_expired_token(user_id, "expired@example.com");

    let request = Request::builder()
        .uri("/api/v1/auth/me")
        .method("GET")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ============================================================================
// Authenticated Happy Path Tests
// ============================================================================

#[tokio::test]
async fn test_get_me_authenticated() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("test-me-{:.8}@example.com", uuid::Uuid::new_v4());
    let tier_name = format!("test-tier-{:.8}", uuid::Uuid::new_v4());
    let (user_id, _, _) = fixtures::create_user_with_subscription(&db, &email, &tier_name)
        .await
        .expect("Failed to create user");

    let token = get_auth_token(user_id, &email).await;

    let request = Request::builder()
        .uri("/api/v1/auth/me")
        .method("GET")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_json_response(response).await;
    assert_eq!(body["email"], email);
    assert_eq!(body["id"], user_id.to_string());

    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

#[tokio::test]
async fn test_update_user_profile() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("test-update-{:.8}@example.com", uuid::Uuid::new_v4());
    let tier_name = format!("test-tier-{:.8}", uuid::Uuid::new_v4());
    let (user_id, _, _) = fixtures::create_user_with_subscription(&db, &email, &tier_name)
        .await
        .expect("Failed to create user");

    let token = get_auth_token(user_id, &email).await;

    let request = Request::builder()
        .uri("/api/v1/users/me")
        .method("PUT")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "displayName": "Updated Name",
                "companyName": "Updated Company"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_json_response(response).await;
    assert_eq!(body["displayName"], "Updated Name");
    assert_eq!(body["companyName"], "Updated Company");
    assert_eq!(body["email"], email);

    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

#[tokio::test]
async fn test_get_and_update_preferences() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("test-prefs-{:.8}@example.com", uuid::Uuid::new_v4());
    let tier_name = format!("test-tier-{:.8}", uuid::Uuid::new_v4());
    let (user_id, _, _) = fixtures::create_user_with_subscription(&db, &email, &tier_name)
        .await
        .expect("Failed to create user");

    let token = get_auth_token(user_id, &email).await;

    // GET preferences
    let get_request = Request::builder()
        .uri("/api/v1/users/me/preferences")
        .method("GET")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let get_response = app.clone().oneshot(get_request).await.unwrap();
    assert_eq!(get_response.status(), StatusCode::OK);

    let prefs = parse_json_response(get_response).await;
    assert!(prefs.get("emailQuotaAlert").is_some());
    assert!(prefs.get("emailSecurityAlert").is_some());
    assert!(prefs.get("emailMarketing").is_some());
    assert!(prefs.get("emailMonthlyReport").is_some());

    // PUT preferences
    let put_request = Request::builder()
        .uri("/api/v1/users/me/preferences")
        .method("PUT")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "emailQuotaAlert": true,
                "emailSecurityAlert": true,
                "emailMarketing": false,
                "emailMonthlyReport": false
            })
            .to_string(),
        ))
        .unwrap();

    let put_response = app.oneshot(put_request).await.unwrap();
    assert_eq!(put_response.status(), StatusCode::OK);

    let updated = parse_json_response(put_response).await;
    assert_eq!(updated["emailQuotaAlert"], true);
    assert_eq!(updated["emailSecurityAlert"], true);
    assert_eq!(updated["emailMarketing"], false);
    assert_eq!(updated["emailMonthlyReport"], false);

    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

#[tokio::test]
async fn test_change_password_authenticated() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("test-chgpw-{:.8}@example.com", uuid::Uuid::new_v4());
    let password = "Original123!";

    // Register user (creates real password hash)
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

    let register_body = parse_json_response(register_response).await;
    let token = register_body["token"].as_str().unwrap().to_string();

    // Change password
    let change_request = Request::builder()
        .uri("/api/v1/auth/change-password")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "currentPassword": password,
                "newPassword": "NewSecure456!"
            })
            .to_string(),
        ))
        .unwrap();

    let change_response = app.clone().oneshot(change_request).await.unwrap();
    assert_eq!(change_response.status(), StatusCode::NO_CONTENT);

    // Login with new password should succeed
    let login_request = Request::builder()
        .uri("/api/v1/auth/login")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "email": &email,
                "password": "NewSecure456!"
            })
            .to_string(),
        ))
        .unwrap();

    let login_response = app.clone().oneshot(login_request).await.unwrap();
    assert_eq!(login_response.status(), StatusCode::OK);

    // Login with old password should fail
    let old_login_request = Request::builder()
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

    let old_login_response = app.oneshot(old_login_request).await.unwrap();
    assert_eq!(old_login_response.status(), StatusCode::UNAUTHORIZED);

    // Clean up
    let _ = sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(&email)
        .execute(db.pool())
        .await;
}

// ============================================================================
// Edge Cases
// ============================================================================

#[tokio::test]
async fn test_update_profile_clear_fields() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("test-clear-{:.8}@example.com", uuid::Uuid::new_v4());
    let user_id = fixtures::create_user_with_profile(&db, &email, Some("Name"), Some("Company"))
        .await
        .expect("Failed to create user");

    // Create subscription for the user
    let tier_name = format!("test-tier-{:.8}", uuid::Uuid::new_v4());
    let tier_id = fixtures::create_tier(
        &db,
        &tier_name,
        "Test",
        pii_redacta_core::db::models::TierLimits::default(),
        pii_redacta_core::db::models::TierFeatures::default(),
    )
    .await
    .expect("Failed to create tier");
    fixtures::create_subscription(
        &db,
        user_id,
        tier_id,
        pii_redacta_core::db::models::SubscriptionStatus::Trial,
    )
    .await
    .expect("Failed to create subscription");

    let token = get_auth_token(user_id, &email).await;

    // Clear display_name by sending empty string
    let request = Request::builder()
        .uri("/api/v1/users/me")
        .method("PUT")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "displayName": "",
                "companyName": "Still Here"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_json_response(response).await;
    assert!(
        body["displayName"].is_null(),
        "Empty string should clear display_name to null"
    );
    assert_eq!(body["companyName"], "Still Here");

    // Clean up
    let _ = sqlx::query("DELETE FROM subscriptions WHERE user_id = $1")
        .bind(user_id)
        .execute(db.pool())
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(db.pool())
        .await;
    let _ = sqlx::query("DELETE FROM tiers WHERE id = $1")
        .bind(tier_id)
        .execute(db.pool())
        .await;
}

#[tokio::test]
async fn test_change_password_wrong_current() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("test-wrongpw-{:.8}@example.com", uuid::Uuid::new_v4());
    let password = "Original123!";

    // Register
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

    let register_body = parse_json_response(register_response).await;
    let token = register_body["token"].as_str().unwrap().to_string();

    // Try to change password with wrong current password
    let change_request = Request::builder()
        .uri("/api/v1/auth/change-password")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "currentPassword": "WrongPassword123!",
                "newPassword": "NewSecure456!"
            })
            .to_string(),
        ))
        .unwrap();

    let change_response = app.oneshot(change_request).await.unwrap();
    assert_eq!(change_response.status(), StatusCode::BAD_REQUEST);

    // Clean up
    let _ = sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(&email)
        .execute(db.pool())
        .await;
}

// ============================================================================
// Token Invalidation After Password Change Tests (H1)
// ============================================================================

/// H1: Old token should be rejected after password change (without Redis, uses DB fallback).
/// Note: These integration tests run without Redis, so only the DB fallback path for
/// token invalidation is exercised. The Redis fast path (`pw_changed:{user_id}`) is
/// tested implicitly via the middleware logic but not in an end-to-end context here.
#[tokio::test]
async fn test_old_token_rejected_after_password_change() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("test-token-inv-{:.8}@example.com", uuid::Uuid::new_v4());
    let password = "Original123!";

    // Register user
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

    let register_body = parse_json_response(register_response).await;
    let old_token = register_body["token"].as_str().unwrap().to_string();

    // Small delay to ensure password_changed_at is strictly after token iat
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;

    // Change password using the old token
    let change_request = Request::builder()
        .uri("/api/v1/auth/change-password")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", old_token))
        .body(Body::from(
            json!({
                "currentPassword": password,
                "newPassword": "NewSecure456!"
            })
            .to_string(),
        ))
        .unwrap();

    let change_response = app.clone().oneshot(change_request).await.unwrap();
    assert_eq!(change_response.status(), StatusCode::NO_CONTENT);

    // Old token should now be rejected
    let me_request = Request::builder()
        .uri("/api/v1/auth/me")
        .method("GET")
        .header("Authorization", format!("Bearer {}", old_token))
        .body(Body::empty())
        .unwrap();

    let me_response = app.clone().oneshot(me_request).await.unwrap();
    assert_eq!(
        me_response.status(),
        StatusCode::UNAUTHORIZED,
        "Old token should be rejected after password change"
    );

    // Clean up
    let _ = sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(&email)
        .execute(db.pool())
        .await;
}

/// H1: New token obtained after password change should work
#[tokio::test]
async fn test_new_token_works_after_password_change() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("test-new-token-{:.8}@example.com", uuid::Uuid::new_v4());
    let password = "Original123!";
    let new_password = "NewSecure456!";

    // Register
    let register_request = Request::builder()
        .uri("/api/v1/auth/register")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({ "email": &email, "password": password }).to_string(),
        ))
        .unwrap();

    let register_response = app.clone().oneshot(register_request).await.unwrap();
    assert_eq!(register_response.status(), StatusCode::OK);
    let register_body = parse_json_response(register_response).await;
    let old_token = register_body["token"].as_str().unwrap().to_string();

    // Small delay
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;

    // Change password
    let change_request = Request::builder()
        .uri("/api/v1/auth/change-password")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", old_token))
        .body(Body::from(
            json!({ "currentPassword": password, "newPassword": new_password }).to_string(),
        ))
        .unwrap();

    let change_response = app.clone().oneshot(change_request).await.unwrap();
    assert_eq!(change_response.status(), StatusCode::NO_CONTENT);

    // Wait so that new token's iat (unix seconds) is strictly after password_changed_at.
    // JWT iat has second-level precision, so tokens issued in the same second as the
    // password change are intentionally rejected (H5 same-second race fix).
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;

    // Login with new password to get new token
    let login_request = Request::builder()
        .uri("/api/v1/auth/login")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({ "email": &email, "password": new_password }).to_string(),
        ))
        .unwrap();

    let login_response = app.clone().oneshot(login_request).await.unwrap();
    assert_eq!(login_response.status(), StatusCode::OK);
    let login_body = parse_json_response(login_response).await;
    let new_token = login_body["token"].as_str().unwrap().to_string();

    // New token should work
    let me_request = Request::builder()
        .uri("/api/v1/auth/me")
        .method("GET")
        .header("Authorization", format!("Bearer {}", new_token))
        .body(Body::empty())
        .unwrap();

    let me_response = app.clone().oneshot(me_request).await.unwrap();
    assert_eq!(
        me_response.status(),
        StatusCode::OK,
        "New token should be accepted after password change"
    );

    // Clean up
    let _ = sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(&email)
        .execute(db.pool())
        .await;
}

/// H1: Verify password_changed_at column exists
#[tokio::test]
async fn test_password_changed_at_column_exists() {
    let db = setup_db().await;

    let result = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_name = 'users' AND column_name = 'password_changed_at'
        )
        "#,
    )
    .fetch_one(db.pool())
    .await
    .expect("Query should succeed");

    assert!(
        result,
        "password_changed_at column should exist in users table"
    );
}

// ============================================================================
// Admin Middleware Tests (H2)
// ============================================================================

/// H2: Non-admin users should be rejected from admin routes
#[tokio::test]
async fn test_non_admin_rejected_from_admin_route() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("test-nonadmin-{:.8}@example.com", uuid::Uuid::new_v4());
    let tier_name = format!("test-tier-{:.8}", uuid::Uuid::new_v4());
    let (user_id, _, _) = fixtures::create_user_with_subscription(&db, &email, &tier_name)
        .await
        .expect("Failed to create user");

    let token = get_auth_token(user_id, &email).await;

    let request = Request::builder()
        .uri("/api/v1/admin/stats")
        .method("GET")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "Non-admin should be rejected from admin routes"
    );

    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

/// H2: Admin users should access admin routes — proves DB verification by using
/// a JWT token with is_admin=false while DB has is_admin=true. The middleware
/// should check the DB (not the JWT claim) and grant access.
#[tokio::test]
async fn test_admin_user_accesses_admin_route() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("test-admin-{:.8}@example.com", uuid::Uuid::new_v4());
    let tier_name = format!("test-tier-{:.8}", uuid::Uuid::new_v4());
    let (user_id, _, _) = fixtures::create_user_with_subscription(&db, &email, &tier_name)
        .await
        .expect("Failed to create user");

    // Promote to admin in DB
    sqlx::query("UPDATE users SET is_admin = true WHERE id = $1")
        .bind(user_id)
        .execute(db.pool())
        .await
        .expect("Failed to promote user to admin");

    // Generate token with is_admin=FALSE in JWT — the middleware should check DB, not JWT
    let config = JwtConfig::new(test_jwt_secret(), 24).expect("Valid JWT config");
    let token = generate_token(user_id, &email, false, &config).expect("Should generate token");

    let request = Request::builder()
        .uri("/api/v1/admin/stats")
        .method("GET")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Admin middleware should verify against DB, not JWT claim"
    );

    let body = parse_json_response(response).await;
    assert!(body.get("totalUsers").is_some());
    assert!(body.get("totalApiKeys").is_some());

    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

/// H2: Unauthenticated requests should be rejected from admin routes
#[tokio::test]
async fn test_admin_route_requires_auth() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/api/v1/admin/stats")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
