//! End-to-End & Cross-Cutting Integration Tests
//!
//! Covers: multi-step user flows, CORS, security headers, concurrency,
//! database connectivity, dashboard, usage, subscription, and tiers.
//!
//! Run with: cargo test --test e2e_test

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
// Multi-Step User Flows
// ============================================================================

/// E2E: Complete user registration and login flow
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
                "displayName": "E2E Test User",
                "companyName": "E2E Test Company"
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

/// E2E: Register → login → GET /me → verify full user data
#[tokio::test]
async fn test_register_login_me_flow() {
    let app = setup_app().await;
    let db = setup_db().await;
    let email = format!("e2e-me-{:.8}@example.com", uuid::Uuid::new_v4());
    let password = "FlowTest123!";

    // Register
    let register_request = Request::builder()
        .uri("/api/v1/auth/register")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "email": &email,
                "password": password,
                "displayName": "Flow User",
                "companyName": "Flow Corp"
            })
            .to_string(),
        ))
        .unwrap();

    let register_response = app.clone().oneshot(register_request).await.unwrap();
    assert_eq!(register_response.status(), StatusCode::OK);

    let register_body = parse_json_response(register_response).await;
    let token = register_body["token"].as_str().unwrap();
    let registered_user_id = register_body["user"]["id"].as_str().unwrap();

    // GET /me with token from registration
    let me_request = Request::builder()
        .uri("/api/v1/auth/me")
        .method("GET")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let me_response = app.clone().oneshot(me_request).await.unwrap();
    assert_eq!(me_response.status(), StatusCode::OK);

    let me_body = parse_json_response(me_response).await;
    assert_eq!(me_body["id"], registered_user_id);
    assert_eq!(me_body["email"], email);
    assert_eq!(me_body["displayName"], "Flow User");
    assert_eq!(me_body["companyName"], "Flow Corp");

    // Clean up
    let _ = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(uuid::Uuid::parse_str(registered_user_id).unwrap())
        .execute(db.pool())
        .await;
}

// ============================================================================
// Infrastructure & CORS Tests
// ============================================================================

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

// ============================================================================
// Public Endpoints
// ============================================================================

#[tokio::test]
async fn test_list_tiers_public() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/api/v1/tiers")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_json_response(response).await;
    assert!(body.is_array(), "Expected array of tiers");

    let tiers = body.as_array().unwrap();
    assert!(!tiers.is_empty(), "Should have at least one tier");

    // Each tier should have expected fields
    let first = &tiers[0];
    assert!(first.get("name").is_some(), "Tier should have name");
    assert!(first.get("limits").is_some(), "Tier should have limits");
}

// ============================================================================
// 401 Rejection Tests — Dashboard, Usage, Subscription
// ============================================================================

#[tokio::test]
async fn test_dashboard_stats_requires_auth() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/api/v1/dashboard/stats")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_dashboard_rejects_expired_token() {
    let app = setup_app().await;
    let user_id = uuid::Uuid::new_v4();
    let token = get_expired_token(user_id, "expired@example.com");

    let request = Request::builder()
        .uri("/api/v1/dashboard/stats")
        .method("GET")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_usage_stats_requires_auth() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/api/v1/usage/stats")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_usage_daily_requires_auth() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/api/v1/usage/daily")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_usage_summary_requires_auth() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/api/v1/usage/summary")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_subscription_requires_auth() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/api/v1/subscription")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_subscription_rejects_invalid_token() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/api/v1/subscription")
        .method("GET")
        .header("Authorization", "Bearer invalid.token.here")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ============================================================================
// Authenticated Dashboard, Usage, Subscription Tests
// ============================================================================

#[tokio::test]
async fn test_usage_stats_authenticated() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("test-usage-{:.8}@example.com", uuid::Uuid::new_v4());
    let tier_name = format!("test-tier-{:.8}", uuid::Uuid::new_v4());
    let (user_id, _, _) = fixtures::create_user_with_subscription(&db, &email, &tier_name)
        .await
        .expect("Failed to create user");

    let token = get_auth_token(user_id, &email).await;

    let request = Request::builder()
        .uri("/api/v1/usage/stats")
        .method("GET")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_json_response(response).await;
    assert!(body.get("totalRequests").is_some());
    assert!(body.get("totalFiles").is_some());
    assert!(body.get("totalPages").is_some());
    assert!(body.get("storageUsed").is_some());
    assert!(body.get("monthlyFiles").is_some());

    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

#[tokio::test]
async fn test_usage_daily_authenticated() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("test-daily-{:.8}@example.com", uuid::Uuid::new_v4());
    let tier_name = format!("test-tier-{:.8}", uuid::Uuid::new_v4());
    let (user_id, _, _) = fixtures::create_user_with_subscription(&db, &email, &tier_name)
        .await
        .expect("Failed to create user");

    let token = get_auth_token(user_id, &email).await;

    let request = Request::builder()
        .uri("/api/v1/usage/daily?days=7")
        .method("GET")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_json_response(response).await;
    assert!(body.is_array(), "Expected array of daily usage entries");

    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

#[tokio::test]
async fn test_usage_summary_authenticated() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("test-summary-{:.8}@example.com", uuid::Uuid::new_v4());
    let tier_name = format!("test-tier-{:.8}", uuid::Uuid::new_v4());
    let (user_id, _, _) = fixtures::create_user_with_subscription(&db, &email, &tier_name)
        .await
        .expect("Failed to create user");

    let token = get_auth_token(user_id, &email).await;

    let request = Request::builder()
        .uri("/api/v1/usage/summary")
        .method("GET")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_json_response(response).await;
    assert!(
        body.get("summary").is_some(),
        "Response should have 'summary' field"
    );
    assert!(
        body.get("dailyUsage").is_some(),
        "Response should have 'dailyUsage' field"
    );
    let summary = &body["summary"];
    assert!(summary.get("monthlyRequests").is_some());
    assert!(summary.get("monthlyDocuments").is_some());
    assert!(summary.get("quotaUsage").is_some());
    assert!(summary.get("documentsChange").is_some());
    assert!(summary.get("requestsChange").is_some());
    assert!(summary.get("quotaUsageChange").is_some());

    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

#[tokio::test]
async fn test_dashboard_stats_authenticated() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("test-dash-{:.8}@example.com", uuid::Uuid::new_v4());
    let tier_name = format!("test-tier-{:.8}", uuid::Uuid::new_v4());
    let (user_id, _, _) = fixtures::create_user_with_subscription(&db, &email, &tier_name)
        .await
        .expect("Failed to create user");

    let token = get_auth_token(user_id, &email).await;

    let request = Request::builder()
        .uri("/api/v1/dashboard/stats")
        .method("GET")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_json_response(response).await;
    assert!(body.get("stats").is_some());
    assert!(body.get("charts").is_some());
    assert!(body.get("recentActivity").is_some());

    let stats = &body["stats"];
    assert!(stats.get("monthlyRequests").is_some());
    assert!(stats.get("monthlyDocuments").is_some());
    assert!(stats.get("quotaUsage").is_some());

    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

#[tokio::test]
async fn test_subscription_authenticated() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("test-sub-{:.8}@example.com", uuid::Uuid::new_v4());
    let tier_name = format!("test-tier-{:.8}", uuid::Uuid::new_v4());
    let (user_id, _, _) = fixtures::create_user_with_subscription(&db, &email, &tier_name)
        .await
        .expect("Failed to create user");

    let token = get_auth_token(user_id, &email).await;

    let request = Request::builder()
        .uri("/api/v1/subscription")
        .method("GET")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_json_response(response).await;
    assert!(body.get("id").is_some());
    assert!(body.get("status").is_some());
    assert!(body.get("tier").is_some());
    assert_eq!(body["status"], "trial");
    assert!(body["tier"].get("name").is_some());
    assert!(body["tier"].get("limits").is_some());

    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

// ============================================================================
// Authenticated Detection Tests
// ============================================================================

#[tokio::test]
async fn test_detect_requires_auth() {
    let app = setup_app().await;

    let request = Request::builder()
        .uri("/api/v1/detect")
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
async fn test_detect_authenticated_success() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("test-detect-{:.8}@example.com", uuid::Uuid::new_v4());
    let tier_name = format!("test-tier-{:.8}", uuid::Uuid::new_v4());
    let (user_id, _, _) = fixtures::create_user_with_subscription(&db, &email, &tier_name)
        .await
        .expect("Failed to create user");

    let token = get_auth_token(user_id, &email).await;

    let request = Request::builder()
        .uri("/api/v1/detect")
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
    assert!(body.get("processing_time_ms").is_some());

    let entities = body["entities"].as_array().unwrap();
    assert!(!entities.is_empty(), "Should detect at least one entity");

    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

#[tokio::test]
async fn test_detect_authenticated_empty_text() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("test-detect-empty-{:.8}@example.com", uuid::Uuid::new_v4());
    let tier_name = format!("test-tier-{:.8}", uuid::Uuid::new_v4());
    let (user_id, _, _) = fixtures::create_user_with_subscription(&db, &email, &tier_name)
        .await
        .expect("Failed to create user");

    let token = get_auth_token(user_id, &email).await;

    let request = Request::builder()
        .uri("/api/v1/detect")
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
async fn test_detect_authenticated_with_redaction() {
    let app = setup_app().await;
    let db = setup_db().await;

    let email = format!("test-detect-redact-{:.8}@example.com", uuid::Uuid::new_v4());
    let tier_name = format!("test-tier-{:.8}", uuid::Uuid::new_v4());
    let (user_id, _, _) = fixtures::create_user_with_subscription(&db, &email, &tier_name)
        .await
        .expect("Failed to create user");

    let token = get_auth_token(user_id, &email).await;

    let request = Request::builder()
        .uri("/api/v1/detect")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "text": "Contact me at john@example.com",
                "options": { "redact": true, "tenant_id": "test-tenant" }
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_json_response(response).await;
    assert!(body.get("redacted_text").is_some());
    let redacted = body["redacted_text"].as_str().unwrap();
    assert!(
        !redacted.contains("john@example.com"),
        "Redacted text should not contain original email"
    );

    fixtures::cleanup_test_data(&db, &[user_id]).await;
}

// ============================================================================
// Rate Limiting Tests
// ============================================================================

#[tokio::test]
async fn test_login_rate_limiting() {
    let app = setup_app().await;
    let test_ip = format!("10.0.0.{}", uuid::Uuid::new_v4().as_bytes()[0]);

    // Send 11 requests (limit is 10/minute) — all will get 401 (bad creds)
    // but the 11th should get 429 (rate limited)
    for i in 0..11 {
        let request = Request::builder()
            .uri("/api/v1/auth/login")
            .method("POST")
            .header("Content-Type", "application/json")
            .header("X-Forwarded-For", &test_ip)
            .body(Body::from(
                json!({
                    "email": "nobody@example.com",
                    "password": "wrong"
                })
                .to_string(),
            ))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        if i < 10 {
            // First 10 requests should get through (even if auth fails)
            assert_ne!(
                response.status(),
                StatusCode::TOO_MANY_REQUESTS,
                "Request {} should not be rate limited",
                i
            );
        } else {
            // 11th request should be rate limited
            assert_eq!(
                response.status(),
                StatusCode::TOO_MANY_REQUESTS,
                "Request {} should be rate limited",
                i
            );
        }
    }
}

#[tokio::test]
async fn test_register_rate_limiting() {
    let app = setup_app().await;
    let test_ip = format!("10.0.1.{}", uuid::Uuid::new_v4().as_bytes()[0]);

    // Send 11 register requests from same IP
    for i in 0..11 {
        let request = Request::builder()
            .uri("/api/v1/auth/register")
            .method("POST")
            .header("Content-Type", "application/json")
            .header("X-Forwarded-For", &test_ip)
            .body(Body::from(
                json!({
                    "email": format!("rate-limit-test-{}@example.com", i),
                    "password": "SecurePass123!"
                })
                .to_string(),
            ))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        if i < 10 {
            assert_ne!(
                response.status(),
                StatusCode::TOO_MANY_REQUESTS,
                "Request {} should not be rate limited",
                i
            );
        } else {
            assert_eq!(
                response.status(),
                StatusCode::TOO_MANY_REQUESTS,
                "Request {} should be rate limited",
                i
            );
        }
    }

    // Clean up registered users
    let db = setup_db().await;
    for i in 0..10 {
        let _ = sqlx::query("DELETE FROM users WHERE email = $1")
            .bind(format!("rate-limit-test-{}@example.com", i))
            .execute(db.pool())
            .await;
    }
}
