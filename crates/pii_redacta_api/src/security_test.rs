//! Security middleware tests
//!
//! Sprint 8: Security Hardening & MVP Release

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::util::ServiceExt;

use crate::create_app;

#[tokio::test]
async fn test_response_has_security_headers() {
    let app = create_app();

    let response = ServiceExt::<Request<Body>>::oneshot(
        app,
        Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap();

    // Check security headers
    assert!(response.headers().contains_key("x-content-type-options"));
    assert_eq!(
        response.headers().get("x-content-type-options").unwrap(),
        "nosniff"
    );
}

#[tokio::test]
async fn test_response_has_x_frame_options() {
    let app = create_app();

    let response = ServiceExt::<Request<Body>>::oneshot(
        app,
        Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap();

    assert!(response.headers().contains_key("x-frame-options"));
}

#[tokio::test]
async fn test_large_request_rejected() {
    let app = create_app();

    // Create a request with large Content-Length
    let large_body = vec![b'x'; 15 * 1024 * 1024];

    let response = ServiceExt::<Request<Body>>::oneshot(
        app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/detect")
            .header("content-type", "application/json")
            .header("content-length", large_body.len())
            .body(Body::from(large_body))
            .unwrap(),
    )
    .await
    .unwrap();

    // Should be rejected with payload too large
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn test_normal_request_accepted() {
    let app = create_app();

    // Create a 1KB body (well under limit)
    let normal_body = r#"{"text": "test@example.com"}"#;

    let response = ServiceExt::<Request<Body>>::oneshot(
        app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/detect")
            .header("content-type", "application/json")
            .header("content-length", normal_body.len())
            .body(Body::from(normal_body))
            .unwrap(),
    )
    .await
    .unwrap();

    // Should be accepted
    assert_eq!(response.status(), StatusCode::OK);
}
