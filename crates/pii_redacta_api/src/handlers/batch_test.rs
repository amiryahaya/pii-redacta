//! Unit tests for batch processing handler types

use super::*;

#[test]
fn test_batch_error_status_codes() {
    let test_cases = vec![
        (BatchError::NotAvailable, StatusCode::FORBIDDEN),
        (
            BatchError::LimitExceeded(100),
            StatusCode::TOO_MANY_REQUESTS,
        ),
        (BatchError::NotFound, StatusCode::NOT_FOUND),
        (BatchError::EmptyItems, StatusCode::BAD_REQUEST),
    ];

    for (error, expected_status) in test_cases {
        let response = error.into_response();
        assert_eq!(response.status(), expected_status);
    }
}

#[test]
fn test_batch_response_serialization() {
    let response = BatchResponse {
        batch_id: "test-batch-id".to_string(),
        status: "pending".to_string(),
        total_items: 5,
    };

    let json = serde_json::to_value(&response).unwrap();
    assert_eq!(json["batchId"], "test-batch-id");
    assert_eq!(json["status"], "pending");
    assert_eq!(json["totalItems"], 5);
}

#[test]
fn test_batch_result_serialization() {
    let item = BatchResultItem {
        item_index: 0,
        status: "completed".to_string(),
        entities: Some(serde_json::json!([{"entity_type": "EMAIL", "value": "test@example.com"}])),
        redacted_text: None,
        processing_time_ms: Some(15),
        error_message: None,
    };

    let json = serde_json::to_value(&item).unwrap();
    assert_eq!(json["itemIndex"], 0);
    assert_eq!(json["status"], "completed");
    assert_eq!(json["processingTimeMs"], 15);
    assert!(json["entities"].is_array());
}

#[test]
fn test_batch_status_response_serialization() {
    let response = BatchStatusResponse {
        id: "batch-123".to_string(),
        status: "completed".to_string(),
        total_items: 3,
        completed_items: 3,
        failed_items: 0,
        redact: false,
        use_custom_rules: false,
        created_at: "2026-03-10T00:00:00Z".to_string(),
        completed_at: Some("2026-03-10T00:00:01Z".to_string()),
    };

    let json = serde_json::to_value(&response).unwrap();
    assert_eq!(json["totalItems"], 3);
    assert_eq!(json["completedItems"], 3);
    assert_eq!(json["failedItems"], 0);
    assert_eq!(json["useCustomRules"], false);
}

#[test]
fn test_batch_request_deserialization() {
    let json = serde_json::json!({
        "items": ["text1", "text2"],
        "redact": true,
        "useCustomRules": false,
    });

    let request: BatchRequest = serde_json::from_value(json).unwrap();
    assert_eq!(request.items.len(), 2);
    assert_eq!(request.redact, Some(true));
    assert_eq!(request.use_custom_rules, Some(false));
}
