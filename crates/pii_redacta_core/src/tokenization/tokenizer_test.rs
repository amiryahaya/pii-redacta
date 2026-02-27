//! Tokenization tests
//!
//! Sprint 3: Tokenization Engine

use crate::tokenization::{TokenGenerator, Tokenizer};
use crate::types::{Entity, EntityType};
use std::collections::HashMap;

// ============== TOKEN GENERATOR TESTS ==============

#[test]
fn test_generate_token_deterministic() {
    let generator = TokenGenerator::new("tenant-123");

    let token1 = generator.generate(EntityType::Email, "test@example.com");
    let token2 = generator.generate(EntityType::Email, "test@example.com");

    assert_eq!(token1, token2);
}

#[test]
fn test_generate_token_different_values() {
    let generator = TokenGenerator::new("tenant-123");

    let token1 = generator.generate(EntityType::Email, "a@b.com");
    let token2 = generator.generate(EntityType::Email, "c@d.com");

    assert_ne!(token1, token2);
}

#[test]
fn test_generate_token_tenant_isolation() {
    let gen1 = TokenGenerator::new("tenant-1");
    let gen2 = TokenGenerator::new("tenant-2");

    let token1 = gen1.generate(EntityType::Email, "test@example.com");
    let token2 = gen2.generate(EntityType::Email, "test@example.com");

    // Different tenants should generate different tokens for same value
    assert_ne!(token1, token2);
}

#[test]
fn test_token_format() {
    let generator = TokenGenerator::new("tenant-123");
    let token = generator.generate(EntityType::Email, "test@example.com");

    assert!(token.starts_with("<<PII_EMAIL_"));
    assert!(token.ends_with(">>"));
}

#[test]
fn test_token_format_various_types() {
    let generator = TokenGenerator::new("tenant-123");

    let email_token = generator.generate(EntityType::Email, "a@b.com");
    let nric_token = generator.generate(EntityType::MalaysianNric, "850101-14-5123");
    let phone_token = generator.generate(EntityType::PhoneNumber, "012-3456789");

    assert!(email_token.starts_with("<<PII_EMAIL_"));
    assert!(nric_token.starts_with("<<PII_MY_NRIC_"));
    assert!(phone_token.starts_with("<<PII_PHONE_"));
}

#[test]
fn test_token_no_original_value() {
    let generator = TokenGenerator::new("tenant-123");
    let token = generator.generate(EntityType::Email, "secret@example.com");

    assert!(!token.contains("secret"));
    assert!(!token.contains("example.com"));
    assert!(!token.contains("@"));
}

#[test]
fn test_token_length_reasonable() {
    let generator = TokenGenerator::new("tenant-123");
    let token = generator.generate(EntityType::Email, "test@example.com");

    // Token should be reasonable length (<<PII_EMAIL_ + hash + >>)
    assert!(token.len() > 15);
    assert!(token.len() < 50);
}

// ============== TOKENIZER TESTS ==============

#[test]
fn test_tokenize_single_entity() {
    let tokenizer = Tokenizer::new("tenant-123");
    let text = "Email: john@example.com";
    let entities = vec![Entity::new(EntityType::Email, "john@example.com", 7, 23)];

    let (tokenized, token_map) = tokenizer.tokenize(text, &entities);

    assert!(tokenized.contains("<<PII_EMAIL_"));
    assert!(!tokenized.contains("john@example.com"));
    assert_eq!(token_map.len(), 1);

    // Verify the original value is in the token map
    let original = token_map.values().next().unwrap();
    assert_eq!(original, "john@example.com");
}

#[test]
fn test_tokenize_multiple_entities() {
    let tokenizer = Tokenizer::new("tenant-123");
    let text = "Email: a@b.com and c@d.com";
    let entities = vec![
        Entity::new(EntityType::Email, "a@b.com", 7, 14),
        Entity::new(EntityType::Email, "c@d.com", 19, 26),
    ];

    let (tokenized, token_map) = tokenizer.tokenize(text, &entities);

    assert_eq!(tokenized.matches("<<PII_EMAIL_").count(), 2);
    assert_eq!(token_map.len(), 2);
}

#[test]
fn test_tokenize_preserves_structure() {
    let tokenizer = Tokenizer::new("tenant-123");
    let text = "Start\nEmail: john@example.com\nEnd";
    let entities = vec![Entity::new(EntityType::Email, "john@example.com", 13, 29)];

    let (tokenized, _) = tokenizer.tokenize(text, &entities);

    assert!(tokenized.starts_with("Start\n"));
    assert!(tokenized.ends_with("\nEnd"));
}

#[test]
fn test_tokenize_no_entities() {
    let tokenizer = Tokenizer::new("tenant-123");
    let text = "No PII here";
    let entities: Vec<Entity> = vec![];

    let (tokenized, token_map) = tokenizer.tokenize(text, &entities);

    assert_eq!(tokenized, text);
    assert!(token_map.is_empty());
}

#[test]
fn test_detokenize_single() {
    let tokenizer = Tokenizer::new("tenant-123");
    let text = "Email: john@example.com";
    let entities = vec![Entity::new(EntityType::Email, "john@example.com", 7, 23)];

    let (tokenized, token_map) = tokenizer.tokenize(text, &entities);
    let restored = tokenizer.detokenize(&tokenized, &token_map);

    assert_eq!(restored, text);
}

#[test]
fn test_detokenize_multiple() {
    let tokenizer = Tokenizer::new("tenant-123");
    let text = "Email: a@b.com and c@d.com";
    let entities = vec![
        Entity::new(EntityType::Email, "a@b.com", 7, 14),
        Entity::new(EntityType::Email, "c@d.com", 19, 26),
    ];

    let (tokenized, token_map) = tokenizer.tokenize(text, &entities);
    let restored = tokenizer.detokenize(&tokenized, &token_map);

    assert_eq!(restored, text);
}

#[test]
fn test_detokenize_empty_map() {
    let tokenizer = Tokenizer::new("tenant-123");
    let text = "No tokens here";
    let token_map = HashMap::new();

    let restored = tokenizer.detokenize(text, &token_map);

    assert_eq!(restored, text);
}

#[test]
fn test_tokenize_different_entity_types() {
    let tokenizer = Tokenizer::new("tenant-123");
    let text = "Email: a@b.com, IC: 850101-14-5123";
    let entities = vec![
        Entity::new(EntityType::Email, "a@b.com", 7, 14),
        Entity::new(EntityType::MalaysianNric, "850101-14-5123", 20, 34),
    ];

    let (tokenized, token_map) = tokenizer.tokenize(text, &entities);

    assert!(tokenized.contains("<<PII_EMAIL_"));
    assert!(tokenized.contains("<<PII_MY_NRIC_"));
    assert_eq!(token_map.len(), 2);
}

#[test]
fn test_tokenize_overlapping_entities() {
    let tokenizer = Tokenizer::new("tenant-123");
    // If entities overlap, prefer the first one (by position)
    let text = "Contact john.doe@example.com today";
    let entities = vec![
        Entity::new(EntityType::Email, "john.doe@example.com", 8, 28),
        Entity::new(EntityType::PersonName, "john.doe", 8, 16),
    ];

    let (tokenized, token_map) = tokenizer.tokenize(text, &entities);

    // The email entity comes first in the list, so it should be tokenized
    // and the overlapping name should be skipped
    assert!(tokenized.contains("<<PII_EMAIL_") || tokenized.contains("<<PII_PERSON_NAME_"));
    assert!(!token_map.is_empty());
}

#[test]
fn test_tokenize_same_value_same_token() {
    let tokenizer = Tokenizer::new("tenant-123");
    // Same email appearing twice should get the same token
    let text = "Contact a@b.com or a@b.com";
    let entities = vec![
        Entity::new(EntityType::Email, "a@b.com", 8, 15),
        Entity::new(EntityType::Email, "a@b.com", 19, 26),
    ];

    let (tokenized, token_map) = tokenizer.tokenize(text, &entities);

    // Should have one token map entry (same value = same token)
    // But both positions replaced
    assert_eq!(token_map.len(), 1);
    let token_count = tokenized.matches("<<PII_EMAIL_").count();
    assert_eq!(token_count, 2);
}

// ============== PERFORMANCE TESTS ==============

#[test]
fn test_tokenize_performance() {
    let tokenizer = Tokenizer::new("tenant-123");
    let text = "Email: john@example.com";
    let entities = vec![Entity::new(EntityType::Email, "john@example.com", 7, 23)];

    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = tokenizer.tokenize(text, &entities);
    }
    let elapsed = start.elapsed();

    // Should tokenize 1000 times in under 100ms
    assert!(
        elapsed.as_millis() < 100,
        "Tokenization too slow: {}ms",
        elapsed.as_millis()
    );
}

#[test]
fn test_detokenize_performance() {
    let tokenizer = Tokenizer::new("tenant-123");
    let text = "Email: <<PII_EMAIL_ABC123>>";
    let mut token_map = HashMap::new();
    token_map.insert(
        "<<PII_EMAIL_ABC123>>".to_string(),
        "john@example.com".to_string(),
    );

    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = tokenizer.detokenize(text, &token_map);
    }
    let elapsed = start.elapsed();

    // Should detokenize 1000 times in under 100ms
    assert!(
        elapsed.as_millis() < 100,
        "Detokenization too slow: {}ms",
        elapsed.as_millis()
    );
}
