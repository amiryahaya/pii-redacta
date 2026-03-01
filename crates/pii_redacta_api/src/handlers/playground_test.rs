//! Unit tests for playground handler types and error codes

use super::*;
use axum::http::StatusCode;
use axum::response::IntoResponse;

#[test]
fn test_playground_error_status_codes() {
    let cases = [
        (PlaygroundError::NotAvailable, StatusCode::FORBIDDEN),
        (
            PlaygroundError::DailyLimitReached,
            StatusCode::TOO_MANY_REQUESTS,
        ),
        (PlaygroundError::FileTooLarge, StatusCode::PAYLOAD_TOO_LARGE),
        (PlaygroundError::TextTooLong, StatusCode::PAYLOAD_TOO_LARGE),
        (PlaygroundError::EmptyInput, StatusCode::BAD_REQUEST),
        (
            PlaygroundError::UnsupportedFileType("image/png".to_string()),
            StatusCode::BAD_REQUEST,
        ),
        (
            PlaygroundError::ExtractionFailed("parse error".to_string()),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
    ];

    for (error, expected_status) in cases {
        let response = error.into_response();
        assert_eq!(
            response.status(),
            expected_status,
            "Wrong status for error variant"
        );
    }
}

/// H4: ExtractionFailed should NOT leak internal details to the client
#[tokio::test]
async fn test_extraction_failed_does_not_leak_details() {
    let error = PlaygroundError::ExtractionFailed("internal: /tmp/file parse error".to_string());
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let message = json["error"]["message"].as_str().unwrap();
    // Should return a generic message, not the internal detail
    assert_eq!(message, "Failed to extract text from file");
    assert!(!message.contains("/tmp/"));
    assert!(!message.contains("internal"));
}

#[test]
fn test_playground_response_serialization() {
    let response = PlaygroundResponse {
        entities: vec![],
        processing_time_ms: 1.5,
        redacted_text: Some("redacted".to_string()),
        text_length: 42,
        daily_usage: PlaygroundUsage {
            used_today: 3,
            daily_limit: Some(10),
        },
    };

    let json = serde_json::to_value(&response).unwrap();
    // Verify camelCase field names
    assert!(json.get("processingTimeMs").is_some());
    assert!(json.get("redactedText").is_some());
    assert!(json.get("textLength").is_some());
    assert!(json.get("dailyUsage").is_some());
    // Verify no snake_case
    assert!(json.get("processing_time_ms").is_none());
    assert!(json.get("redacted_text").is_none());
    assert!(json.get("text_length").is_none());
    assert!(json.get("daily_usage").is_none());
}

#[test]
fn test_playground_usage_serialization() {
    let usage = PlaygroundUsage {
        used_today: 5,
        daily_limit: Some(25),
    };

    let json = serde_json::to_value(&usage).unwrap();
    assert!(json.get("usedToday").is_some());
    assert!(json.get("dailyLimit").is_some());
    assert_eq!(json["usedToday"], 5);
    assert_eq!(json["dailyLimit"], 25);
    // No snake_case
    assert!(json.get("used_today").is_none());
    assert!(json.get("daily_limit").is_none());
}

#[test]
fn test_playground_usage_unlimited() {
    let usage = PlaygroundUsage {
        used_today: 50,
        daily_limit: None,
    };

    let json = serde_json::to_value(&usage).unwrap();
    assert_eq!(json["usedToday"], 50);
    assert!(json["dailyLimit"].is_null());
}

#[test]
fn test_playground_history_serialization() {
    let entry = PlaygroundHistoryEntry {
        id: "test-id".to_string(),
        request_type: "playground".to_string(),
        file_name: None,
        file_type: None,
        detections_count: Some(5),
        processing_time_ms: Some(12),
        success: true,
        created_at: "2026-03-01T00:00:00Z".to_string(),
    };

    let json = serde_json::to_value(&entry).unwrap();
    // Verify camelCase
    assert!(json.get("requestType").is_some());
    assert!(json.get("detectionsCount").is_some());
    assert!(json.get("processingTimeMs").is_some());
    assert!(json.get("createdAt").is_some());
    // Verify no snake_case
    assert!(json.get("request_type").is_none());
    assert!(json.get("detections_count").is_none());
    assert!(json.get("processing_time_ms").is_none());
    assert!(json.get("created_at").is_none());
    // Optional fields not present when None
    assert!(json.get("fileName").is_none());
    assert!(json.get("fileType").is_none());
}

#[test]
fn test_playground_history_with_file() {
    let entry = PlaygroundHistoryEntry {
        id: "test-id".to_string(),
        request_type: "playground_file".to_string(),
        file_name: Some("test.pdf".to_string()),
        file_type: Some("application/pdf".to_string()),
        detections_count: Some(3),
        processing_time_ms: Some(45),
        success: true,
        created_at: "2026-03-01T00:00:00Z".to_string(),
    };

    let json = serde_json::to_value(&entry).unwrap();
    assert_eq!(json["fileName"], "test.pdf");
    assert_eq!(json["fileType"], "application/pdf");
    assert_eq!(json["requestType"], "playground_file");
}

/// H1: text/csv should be accepted as a supported MIME type
#[test]
fn test_csv_mime_type_supported() {
    assert!(is_supported_mime("text/csv"));
    assert!(is_supported_mime("text/plain"));
    assert!(is_supported_mime("application/pdf"));
    assert!(!is_supported_mime("image/png"));
    assert!(!is_supported_mime("application/json"));
}

/// M4: abbreviated OpenXML MIME should be accepted (mapped to DOCX in handler)
#[test]
fn test_abbreviated_openxml_mime_supported() {
    assert!(is_supported_mime("application/vnd.openxmlformats"));
    assert!(is_supported_mime(
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
    ));
    // XLSX-specific MIME should NOT be supported
    assert!(!is_supported_mime(
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
    ));
}

/// M3: file name sanitization
#[test]
fn test_sanitize_file_name_strips_path_separators() {
    assert_eq!(sanitize_file_name("report.pdf"), "report.pdf");
    assert_eq!(sanitize_file_name("../../etc/passwd"), "....etcpasswd");
    assert_eq!(
        sanitize_file_name("C:\\Users\\test\\file.txt"),
        "C:Userstestfile.txt"
    );
}

#[test]
fn test_sanitize_file_name_strips_control_chars() {
    assert_eq!(sanitize_file_name("file\x00name.pdf"), "filename.pdf");
    assert_eq!(sanitize_file_name("file\nname.pdf"), "filename.pdf");
}

#[test]
fn test_sanitize_file_name_truncates_to_255() {
    let long_name = "a".repeat(300);
    let sanitized = sanitize_file_name(&long_name);
    assert_eq!(sanitized.len(), MAX_FILE_NAME_LEN);
}

#[test]
fn test_sanitize_file_name_empty_becomes_unnamed() {
    assert_eq!(sanitize_file_name(""), "unnamed");
    assert_eq!(sanitize_file_name("/"), "unnamed");
}

/// L4: history query params defaults
#[test]
fn test_history_query_defaults() {
    let q: HistoryQuery = serde_json::from_str("{}").unwrap();
    assert_eq!(q.limit, None);
    assert_eq!(q.offset, None);
}
