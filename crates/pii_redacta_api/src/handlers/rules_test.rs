//! Unit tests for custom rules handler types

use super::*;

#[test]
fn test_rule_error_status_codes() {
    // Verify error variants produce correct status codes
    let test_cases = vec![
        (RuleError::NotAvailable, StatusCode::FORBIDDEN),
        (RuleError::LimitReached, StatusCode::TOO_MANY_REQUESTS),
        (
            RuleError::InvalidPattern("bad regex".to_string()),
            StatusCode::BAD_REQUEST,
        ),
        (RuleError::NotFound, StatusCode::NOT_FOUND),
        (
            RuleError::InvalidInput("bad input".to_string()),
            StatusCode::BAD_REQUEST,
        ),
    ];

    for (error, expected_status) in test_cases {
        let response = error.into_response();
        assert_eq!(response.status(), expected_status);
    }
}

#[test]
fn test_rule_response_serialization() {
    let response = RuleResponse {
        id: "test-id".to_string(),
        name: "Employee ID".to_string(),
        description: Some("Detects employee IDs".to_string()),
        pattern: r"EMP-\d{6}".to_string(),
        entity_label: "EMPLOYEE_ID".to_string(),
        confidence: 0.95,
        is_active: true,
        created_at: "2026-03-10T00:00:00Z".to_string(),
        updated_at: "2026-03-10T00:00:00Z".to_string(),
    };

    let json = serde_json::to_value(&response).unwrap();
    assert_eq!(json["name"], "Employee ID");
    assert_eq!(json["entityLabel"], "EMPLOYEE_ID");
    let conf = json["confidence"].as_f64().unwrap();
    assert!((conf - 0.95).abs() < 0.001, "confidence should be ~0.95");
    assert_eq!(json["isActive"], true);
}

#[test]
fn test_create_rule_request_deserialization() {
    let json = serde_json::json!({
        "name": "Test Rule",
        "pattern": r"\d{4}",
        "entityLabel": "TEST",
    });

    let request: CreateRuleRequest = serde_json::from_value(json).unwrap();
    assert_eq!(request.name, "Test Rule");
    assert_eq!(request.entity_label, "TEST");
    assert!(request.confidence.is_none());
    assert!(request.description.is_none());
}
