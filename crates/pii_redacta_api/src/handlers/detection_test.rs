//! Detection handler tests
//!
//! Sprint 4: REST API Foundation

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use pii_redacta_core::detection::PatternDetector;
use std::sync::Arc;

use super::*;

fn create_test_state() -> Arc<PatternDetector> {
    Arc::new(PatternDetector::new())
}

#[tokio::test]
async fn test_detect_simple_email() {
    let state = create_test_state();
    let request = DetectRequest {
        text: "Email: test@example.com".to_string(),
        options: None,
    };

    let (status, Json(body)) = detect(State(state), Json(request)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.entities.len(), 1);
    assert_eq!(body.entities[0].value, "test@example.com");
}

#[tokio::test]
async fn test_detect_multiple_entities() {
    let state = create_test_state();
    let request = DetectRequest {
        text: "Email: a@b.com, IC: 850101-14-5123".to_string(),
        options: None,
    };

    let (status, Json(body)) = detect(State(state), Json(request)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.entities.len(), 2);
}

#[tokio::test]
async fn test_detect_no_entities() {
    let state = create_test_state();
    let request = DetectRequest {
        text: "No PII here at all".to_string(),
        options: None,
    };

    let (status, Json(body)) = detect(State(state), Json(request)).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.entities.is_empty());
}

#[tokio::test]
async fn test_detect_empty_text_bad_request() {
    let state = create_test_state();
    let request = DetectRequest {
        text: "".to_string(),
        options: None,
    };

    let (status, _) = detect(State(state), Json(request)).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_detect_with_redact_option() {
    let state = create_test_state();
    let request = DetectRequest {
        text: "Email: test@example.com".to_string(),
        options: Some(DetectionOptions {
            redact: true,
            tenant_id: Some("test-tenant".to_string()),
        }),
    };

    let (status, Json(body)) = detect(State(state), Json(request)).await;

    assert_eq!(status, StatusCode::OK);
    let redacted = body.redacted_text.expect("redacted_text should be present");
    assert!(redacted.contains("<<PII_EMAIL_"));
    assert!(!redacted.contains("test@example.com"));
}

#[tokio::test]
async fn test_detect_response_has_processing_time() {
    let state = create_test_state();
    let request = DetectRequest {
        text: "Email: test@example.com".to_string(),
        options: None,
    };

    let (_, Json(body)) = detect(State(state), Json(request)).await;

    assert!(body.processing_time_ms > 0.0);
}
