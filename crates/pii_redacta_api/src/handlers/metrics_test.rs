//! Metrics handler tests
//!
//! Sprint 7: Observability & Documentation

use axum::extract::State;
use axum::http::StatusCode;
use std::sync::Arc;

use super::*;
use crate::handlers::JobQueue;

fn create_test_state() -> Arc<JobQueue> {
    Arc::new(JobQueue::new())
}

#[tokio::test]
async fn test_metrics_endpoint_returns_ok() {
    let state = create_test_state();
    let (status, _) = metrics(State(state)).await;

    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_metrics_returns_prometheus_format() {
    let state = create_test_state();
    let (_, body) = metrics(State(state)).await;

    // MVP stub returns static prometheus-style content
    assert!(!body.is_empty(), "metrics body should not be empty");
    assert!(
        body.contains("# HELP") || body.contains("# TYPE") || body.contains("pii_"),
        "metrics body should contain Prometheus markers, got: {body}"
    );
}

#[tokio::test]
async fn test_metrics_contains_detection_counter() {
    let state = create_test_state();
    let (_, body) = metrics(State(state)).await;

    assert!(
        body.contains("pii_detection_requests_total"),
        "metrics body should contain detection counter, got: {body}"
    );
}

#[tokio::test]
async fn test_metrics_contains_upload_counter() {
    let state = create_test_state();
    let (_, body) = metrics(State(state)).await;

    assert!(
        body.contains("pii_files_uploaded_total"),
        "metrics body should contain upload counter, got: {body}"
    );
}
