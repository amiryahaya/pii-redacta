//! Unit tests for webhooks handler types

use super::*;

#[test]
fn test_webhook_error_status_codes() {
    let test_cases = vec![
        (WebhookError::NotAvailable, StatusCode::FORBIDDEN),
        (WebhookError::LimitReached, StatusCode::TOO_MANY_REQUESTS),
        (
            WebhookError::InvalidUrl("bad url".to_string()),
            StatusCode::BAD_REQUEST,
        ),
        (WebhookError::NotFound, StatusCode::NOT_FOUND),
    ];

    for (error, expected_status) in test_cases {
        let response = error.into_response();
        assert_eq!(response.status(), expected_status);
    }
}

#[test]
fn test_webhook_response_serialization() {
    let response = WebhookResponse {
        id: "wh-123".to_string(),
        url: "https://example.com/webhook".to_string(),
        description: Some("Test webhook".to_string()),
        secret: "abc12345...".to_string(),
        events: vec!["detection.completed".to_string()],
        is_active: true,
        failure_count: 0,
        last_triggered_at: None,
        created_at: "2026-03-10T00:00:00Z".to_string(),
    };

    let json = serde_json::to_value(&response).unwrap();
    assert_eq!(json["url"], "https://example.com/webhook");
    assert_eq!(json["isActive"], true);
    assert_eq!(json["failureCount"], 0);
    assert!(json["events"].is_array());
}

#[test]
fn test_validate_webhook_url_valid() {
    assert!(validate_webhook_url("https://example.com/webhook").is_ok());
    assert!(validate_webhook_url("https://hooks.example.com/v1/events").is_ok());
}

#[test]
fn test_validate_webhook_url_http_rejected() {
    let result = validate_webhook_url("http://example.com/webhook");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("HTTPS"));
}

#[test]
fn test_validate_webhook_url_localhost_rejected() {
    assert!(validate_webhook_url("https://localhost/webhook").is_err());
    assert!(validate_webhook_url("https://127.0.0.1/webhook").is_err());
}

#[test]
fn test_validate_webhook_url_private_ip_rejected() {
    assert!(validate_webhook_url("https://10.0.0.1/webhook").is_err());
    assert!(validate_webhook_url("https://192.168.1.1/webhook").is_err());
    assert!(validate_webhook_url("https://172.16.0.1/webhook").is_err());
}

#[test]
fn test_mask_secret() {
    assert_eq!(mask_secret("abcdefghijklmnop"), "abcdefgh...");
    assert_eq!(mask_secret("short"), "***");
}

#[test]
fn test_create_webhook_request_deserialization() {
    let json = serde_json::json!({
        "url": "https://example.com/hook",
        "events": ["detection.completed", "batch.completed"]
    });

    let request: CreateWebhookRequest = serde_json::from_value(json).unwrap();
    assert_eq!(request.url, "https://example.com/hook");
    assert_eq!(request.events.len(), 2);
    assert!(request.description.is_none());
}

#[test]
fn test_webhook_delivery_response_serialization() {
    let delivery = WebhookDeliveryResponse {
        id: "del-1".to_string(),
        event_type: "detection.completed".to_string(),
        status: "delivered".to_string(),
        http_status: Some(200),
        attempts: 1,
        created_at: "2026-03-10T00:00:00Z".to_string(),
        delivered_at: Some("2026-03-10T00:00:01Z".to_string()),
    };

    let json = serde_json::to_value(&delivery).unwrap();
    assert_eq!(json["eventType"], "detection.completed");
    assert_eq!(json["httpStatus"], 200);
}
