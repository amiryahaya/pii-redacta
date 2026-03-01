//! Unit tests for custom rule detection

use super::*;

#[test]
fn test_validate_pattern_valid() {
    assert!(CustomRuleDetector::validate_pattern(r"\d{4}-\d{4}").is_ok());
    assert!(CustomRuleDetector::validate_pattern(r"EMP-\d{6}").is_ok());
    assert!(CustomRuleDetector::validate_pattern(r"[A-Z]{2}\d{8}").is_ok());
}

#[test]
fn test_validate_pattern_invalid() {
    // Unclosed group
    assert!(CustomRuleDetector::validate_pattern(r"(abc").is_err());
    // Invalid escape
    assert!(CustomRuleDetector::validate_pattern(r"\p{Invalid}").is_err());
}

#[test]
fn test_validate_pattern_empty() {
    let result = CustomRuleDetector::validate_pattern("");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("empty"));
}

#[test]
fn test_detect_custom_rules() {
    let detector = CustomRuleDetector::new();
    let rules = vec![CustomRule {
        id: Uuid::new_v4(),
        name: "Employee ID".to_string(),
        pattern: r"EMP-\d{6}".to_string(),
        entity_label: "EMPLOYEE_ID".to_string(),
        confidence: 0.95,
    }];

    let text = "Employee EMP-123456 joined the team.";
    let entities = detector.detect(text, &rules);

    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].entity_type, EntityType::Custom);
    assert_eq!(entities[0].value, "EMP-123456");
    assert_eq!(entities[0].custom_label.as_deref(), Some("EMPLOYEE_ID"));
    assert_eq!(entities[0].confidence, Some(0.95));
    assert_eq!(entities[0].start, 9);
    assert_eq!(entities[0].end, 19);
}

#[test]
fn test_detect_custom_rules_empty() {
    let detector = CustomRuleDetector::new();
    let entities = detector.detect("Hello world", &[]);
    assert!(entities.is_empty());
}

#[test]
fn test_custom_entity_has_label() {
    let entity = Entity::custom("INTERNAL_CODE", "IC-9999", 0, 7, 0.9);
    assert_eq!(entity.entity_type, EntityType::Custom);
    assert_eq!(entity.custom_label.as_deref(), Some("INTERNAL_CODE"));
    assert_eq!(entity.confidence, Some(0.9));
}

#[test]
fn test_pattern_size_limit() {
    // Pattern exceeding max length should be rejected
    let long_pattern = "a".repeat(501);
    let result = CustomRuleDetector::validate_pattern(&long_pattern);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("too long"));
}

#[test]
fn test_multiple_rules_multiple_matches() {
    let detector = CustomRuleDetector::new();
    let rules = vec![
        CustomRule {
            id: Uuid::new_v4(),
            name: "Employee ID".to_string(),
            pattern: r"EMP-\d{6}".to_string(),
            entity_label: "EMPLOYEE_ID".to_string(),
            confidence: 0.95,
        },
        CustomRule {
            id: Uuid::new_v4(),
            name: "Project Code".to_string(),
            pattern: r"PRJ-[A-Z]{3}-\d{3}".to_string(),
            entity_label: "PROJECT_CODE".to_string(),
            confidence: 0.9,
        },
    ];

    let text = "EMP-123456 is working on PRJ-ABC-001 and PRJ-DEF-002.";
    let entities = detector.detect(text, &rules);

    assert_eq!(entities.len(), 3);
    // First rule match
    assert_eq!(entities[0].custom_label.as_deref(), Some("EMPLOYEE_ID"));
    // Second rule matches
    assert_eq!(entities[1].custom_label.as_deref(), Some("PROJECT_CODE"));
    assert_eq!(entities[2].custom_label.as_deref(), Some("PROJECT_CODE"));
}
