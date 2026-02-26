//! Pattern detection tests
//!
//! Sprint 2: Pattern-Based Detection Engine

use crate::detection::PatternDetector;
use crate::types::EntityType;

// ============== EMAIL DETECTION TESTS ==============

#[test]
fn test_detect_email_simple() {
    let detector = PatternDetector::new();
    let text = "Contact john@example.com for details";
    let entities = detector.detect_email(text);

    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].value, "john@example.com");
    assert_eq!(entities[0].entity_type, EntityType::Email);
    assert_eq!(entities[0].start, 8);
    assert_eq!(entities[0].end, 24);
}

#[test]
fn test_detect_email_multiple() {
    let detector = PatternDetector::new();
    let text = "john@example.com and jane@test.org";
    let entities = detector.detect_email(text);

    assert_eq!(entities.len(), 2);
    assert_eq!(entities[0].value, "john@example.com");
    assert_eq!(entities[1].value, "jane@test.org");
}

#[test]
fn test_detect_email_none() {
    let detector = PatternDetector::new();
    let text = "No email here at all";
    let entities = detector.detect_email(text);

    assert!(entities.is_empty());
}

#[test]
fn test_detect_email_with_plus() {
    let detector = PatternDetector::new();
    let text = "user+tag@example.com";
    let entities = detector.detect_email(text);

    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].value, "user+tag@example.com");
}

#[test]
fn test_detect_email_subdomain() {
    let detector = PatternDetector::new();
    let text = "admin@mail.example.co.uk";
    let entities = detector.detect_email(text);

    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].value, "admin@mail.example.co.uk");
}

// ============== MALAYSIAN NRIC TESTS ==============

#[test]
fn test_detect_malaysian_nric_standard() {
    let detector = PatternDetector::new();
    let text = "IC: 850101-14-5123";
    let entities = detector.detect_nric(text);

    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].value, "850101-14-5123");
    assert_eq!(entities[0].entity_type, EntityType::MalaysianNric);
}

#[test]
fn test_detect_malaysian_nric_no_dashes() {
    let detector = PatternDetector::new();
    let text = "IC: 850101145123";
    let entities = detector.detect_nric(text);

    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].value, "850101145123");
}

#[test]
fn test_detect_malaysian_nric_multiple() {
    let detector = PatternDetector::new();
    let text = "IC1: 850101-14-5123, IC2: 900202-13-4122";
    let entities = detector.detect_nric(text);

    assert_eq!(entities.len(), 2);
}

// ============== PHONE NUMBER TESTS ==============

#[test]
fn test_detect_phone_malaysian_mobile() {
    let detector = PatternDetector::new();
    let text = "Call me at 012-3456789";
    let entities = detector.detect_phone(text);

    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].value, "012-3456789");
    assert_eq!(entities[0].entity_type, EntityType::PhoneNumber);
}

#[test]
fn test_detect_phone_international() {
    let detector = PatternDetector::new();
    let text = "Contact +60 12-345 6789";
    let entities = detector.detect_phone(text);

    assert_eq!(entities.len(), 1);
}

#[test]
fn test_detect_phone_no_country_code() {
    let detector = PatternDetector::new();
    let text = "My number is 0123456789";
    let entities = detector.detect_phone(text);

    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].value, "0123456789");
}

// ============== CREDIT CARD TESTS ==============

#[test]
fn test_detect_credit_card_formatted() {
    let detector = PatternDetector::new();
    let text = "Card: 4532-1234-5678-9012";
    let entities = detector.detect_credit_card(text);

    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].value, "4532-1234-5678-9012");
    assert_eq!(entities[0].entity_type, EntityType::CreditCard);
}

#[test]
fn test_detect_credit_card_no_spaces() {
    let detector = PatternDetector::new();
    let text = "Card: 4532123456789012";
    let entities = detector.detect_credit_card(text);

    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].value, "4532123456789012");
}

#[test]
fn test_detect_credit_card_with_spaces() {
    let detector = PatternDetector::new();
    let text = "Card: 4532 1234 5678 9012";
    let entities = detector.detect_credit_card(text);

    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].value, "4532 1234 5678 9012");
}

// ============== DETECT ALL TESTS ==============

#[test]
fn test_detect_all_finds_all_types() {
    let detector = PatternDetector::new();
    let text = "Email: john@example.com, IC: 850101-14-5123, Phone: 012-3456789";
    let entities = detector.detect_all(text);

    assert_eq!(entities.len(), 3);

    let types: Vec<_> = entities.iter().map(|e| e.entity_type).collect();
    assert!(types.contains(&EntityType::Email));
    assert!(types.contains(&EntityType::MalaysianNric));
    assert!(types.contains(&EntityType::PhoneNumber));
}

#[test]
fn test_detect_all_sorted_by_position() {
    let detector = PatternDetector::new();
    let text = "Z: john@example.com, A: 850101-14-5123";
    let entities = detector.detect_all(text);

    // Should be sorted by position in text, not by type
    assert_eq!(entities[0].entity_type, EntityType::Email);
    assert_eq!(entities[1].entity_type, EntityType::MalaysianNric);
}

// ============== PERFORMANCE TESTS ==============

#[test]
fn test_performance_email_detection() {
    let detector = PatternDetector::new();
    let text = "Contact john@example.com or jane@test.org for details";

    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = detector.detect_email(text);
    }
    let elapsed = start.elapsed();

    // Should process 1000 iterations in under 100ms (0.1ms per detection)
    assert!(
        elapsed.as_millis() < 100,
        "Email detection too slow: {}ms",
        elapsed.as_millis()
    );
}

#[test]
fn test_performance_full_detection() {
    let detector = PatternDetector::new();
    let text = "Email: a@b.com, IC: 850101-14-5123, Phone: 012-3456789, Card: 4532-1234-5678-9012";

    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = detector.detect_all(text);
    }
    let elapsed = start.elapsed();

    // Should process 1000 full detections in under 500ms (0.5ms per detection)
    assert!(
        elapsed.as_millis() < 500,
        "Full detection too slow: {}ms",
        elapsed.as_millis()
    );
}

// ============== EDGE CASE TESTS ==============

#[test]
fn test_detect_empty_string() {
    let detector = PatternDetector::new();
    let entities = detector.detect_all("");
    assert!(entities.is_empty());
}

#[test]
fn test_detect_very_long_text() {
    let detector = PatternDetector::new();
    let text = format!("Start {} End", "x".repeat(10000));
    let entities = detector.detect_all(&text);
    assert!(entities.is_empty());
}

#[test]
fn test_detect_confidence_present() {
    let detector = PatternDetector::new();
    let text = "john@example.com";
    let entities = detector.detect_email(text);

    assert_eq!(entities.len(), 1);
    assert!(entities[0].confidence.is_some());
    assert!(entities[0].confidence.unwrap() > 0.0);
    assert!(entities[0].confidence.unwrap() <= 1.0);
}
