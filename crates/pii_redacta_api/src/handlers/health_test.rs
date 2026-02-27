//! Health handler tests
//!
//! Sprint 4: REST API Foundation

use axum::http::StatusCode;

use super::*;

#[tokio::test]
async fn test_health_returns_ok() {
    let (status, _) = health().await;

    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_health_response_has_version() {
    let (_, body) = health().await;

    assert!(!body.version.is_empty());
}

#[tokio::test]
async fn test_health_status_is_healthy() {
    let (_, body) = health().await;

    assert_eq!(body.status, "healthy");
}
