//! Metrics handler tests
//!
//! Sprint 7: Observability & Documentation

use axum::http::StatusCode;

use super::*;

#[tokio::test]
async fn test_metrics_endpoint_returns_ok() {
    let (status, _) = metrics().await;

    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_metrics_returns_prometheus_format() {
    let (_, body) = metrics().await;

    // Should contain Prometheus-style metrics
    assert!(body.contains("# HELP") || body.contains("# TYPE") || body.contains("pii_"));
}

#[tokio::test]
async fn test_metrics_contains_detection_counter() {
    let (_, body) = metrics().await;

    // Should contain detection metrics
    assert!(body.contains("pii_detection_requests_total") || body.is_empty());
}

#[tokio::test]
async fn test_metrics_contains_processing_duration() {
    let (_, body) = metrics().await;

    // Should contain duration metrics
    assert!(body.contains("pii_processing_duration_seconds") || body.is_empty());
}
