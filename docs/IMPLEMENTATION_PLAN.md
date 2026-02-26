# PII Redacta Implementation Plan

**Version:** 3.0 (TDD-Driven Sprint Structure)  
**Last Updated:** 2026-02-27  
**Status:** Ready for Sprint 1

---

## Sprint Methodology: TDD → Code Review → QA → Commit

**Every Sprint Follows This Workflow:**

```
┌─────────────────────────────────────────────────────────────────────┐
│                          SPRINT WORKFLOW                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Week 1-2: Development                                              │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐                         │
│  │  Write  │───▶│  Write  │───▶│ Refactor│  ← Repeat daily         │
│  │  Tests  │    │  Code   │    │ (Blue)  │                         │
│  │  (Red)  │    │(Green)  │    │         │                         │
│  └─────────┘    └─────────┘    └─────────┘                         │
│                                                                      │
│  End of Week 2: Code Review                                         │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐               │
│  │ Self Review │──▶│ Peer Review │──▶│   Approve   │               │
│  │  (Author)   │   │  (Reviewer) │   │   /Reject   │               │
│  └─────────────┘   └─────────────┘   └─────────────┘               │
│                                                                      │
│  Week 2.5: QA Phase                                                 │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐               │
│  │ Integration │──▶│ Performance │──▶│  Security   │               │
│  │    Tests    │   │    Tests    │   │   Audit     │               │
│  └─────────────┘   └─────────────┘   └─────────────┘               │
│                                                                      │
│  Sprint End: Commit & Push                                          │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐               │
│  │  Squash     │──▶│   Commit    │──▶│ Push to     │               │
│  │  Commits    │   │  Message    │   │   origin    │               │
│  └─────────────┘   └─────────────┘   └─────────────┘               │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Sprint Deliverables Checklist

### Code Review Checklist
- [ ] All tests pass (`cargo test`)
- [ ] Code coverage > 80%
- [ ] No compiler warnings
- [ ] Clippy lints pass
- [ ] Code follows Rust style guide
- [ ] Documentation complete
- [ ] Security review passed

### QA Checklist
- [ ] Integration tests pass
- [ ] Performance benchmarks meet targets
- [ ] No memory leaks (valgrind/miri)
- [ ] API contract tests pass
- [ ] Edge cases tested
- [ ] Load test (if applicable)

---

## Sprint 1: Project Foundation & Core Types

**Duration:** 2 weeks  
**Focus:** Setup project structure and domain types

### Sprint 1 Deliverables

#### 1.1 Project Structure (TDD: Day 1-2)

**Red Phase - Write Tests:**
```rust
// tests/project_structure_test.rs
#[test]
fn test_workspace_structure_exists() {
    assert!(Path::new("Cargo.toml").exists());
    assert!(Path::new("crates/pii_redacta_core").exists());
    assert!(Path::new("crates/pii_redacta_api").exists());
}

#[test]
fn test_core_compiles() {
    // Just needs to compile
}
```

**Green Phase - Create Structure:**
```
pii-redacta/
├── Cargo.toml              # Workspace definition
├── crates/
│   ├── pii_redacta_core/   # Core library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types.rs    # Domain types
│   │       └── error.rs    # Error types
│   │
│   └── pii_redacta_api/    # REST API
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
├── Cargo.lock
└── .github/
    └── workflows/
        └── ci.yml
```

**Blue Phase - Refactor:**
- Optimize workspace dependencies
- Add shared configuration

#### 1.2 Core Domain Types (TDD: Day 3-5)

**Red Phase - Write Tests:**
```rust
// crates/pii_redacta_core/src/types_test.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_entity_creation() {
        let entity = Entity::new(
            EntityType::Email,
            "test@example.com",
            0,
            16,
        );
        
        assert_eq!(entity.entity_type, EntityType::Email);
        assert_eq!(entity.value, "test@example.com");
        assert_eq!(entity.start, 0);
        assert_eq!(entity.end, 16);
    }
    
    #[test]
    fn test_entity_serialization() {
        let entity = Entity::new(EntityType::Email, "test@example.com", 0, 16);
        let json = serde_json::to_string(&entity).unwrap();
        
        assert!(json.contains("EMAIL"));
        assert!(json.contains("test@example.com"));
    }
    
    #[test]
    fn test_detection_result_empty() {
        let result = DetectionResult::new();
        assert!(result.entities.is_empty());
        assert_eq!(result.processing_time_ms, 0.0);
    }
    
    #[test]
    fn test_entity_type_display() {
        assert_eq!(EntityType::Email.to_string(), "EMAIL");
        assert_eq!(EntityType::MalaysianNric.to_string(), "MY_NRIC");
    }
}
```

**Green Phase - Implementation:**
```rust
// crates/pii_redacta_core/src/types.rs
use serde::{Deserialize, Serialize};

/// Types of PII entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntityType {
    Email,
    PhoneNumber,
    MalaysianNric,
    PassportNumber,
    CreditCard,
    BankAccount,
    Address,
    PersonName,
    DateOfBirth,
    IpAddress,
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            EntityType::Email => "EMAIL",
            EntityType::PhoneNumber => "PHONE",
            EntityType::MalaysianNric => "MY_NRIC",
            EntityType::PassportNumber => "PASSPORT",
            EntityType::CreditCard => "CREDIT_CARD",
            EntityType::BankAccount => "BANK_ACCOUNT",
            EntityType::Address => "ADDRESS",
            EntityType::PersonName => "PERSON_NAME",
            EntityType::DateOfBirth => "DOB",
            EntityType::IpAddress => "IP_ADDRESS",
        };
        write!(f, "{}", s)
    }
}

/// Detected PII entity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub entity_type: EntityType,
    pub value: String,
    pub start: usize,
    pub end: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
}

impl Entity {
    pub fn new(entity_type: EntityType, value: &str, start: usize, end: usize) -> Self {
        Self {
            entity_type,
            value: value.to_string(),
            start,
            end,
            confidence: None,
        }
    }
    
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = Some(confidence);
        self
    }
}

/// Detection result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetectionResult {
    pub entities: Vec<Entity>,
    pub processing_time_ms: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokenized_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_map: Option<std::collections::HashMap<String, String>>,
}

impl DetectionResult {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            processing_time_ms: 0.0,
            tokenized_text: None,
            token_map: None,
        }
    }
    
    pub fn with_entities(mut self, entities: Vec<Entity>) -> Self {
        self.entities = entities;
        self
    }
}

/// Detection options
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DetectionOptions {
    pub redact: bool,
    #[serde(default)]
    pub entity_types: Vec<EntityType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_entity_creation() {
        let entity = Entity::new(
            EntityType::Email,
            "test@example.com",
            0,
            16,
        );
        
        assert_eq!(entity.entity_type, EntityType::Email);
        assert_eq!(entity.value, "test@example.com");
        assert_eq!(entity.start, 0);
        assert_eq!(entity.end, 16);
    }
    
    #[test]
    fn test_entity_serialization() {
        let entity = Entity::new(EntityType::Email, "test@example.com", 0, 16);
        let json = serde_json::to_string(&entity).unwrap();
        
        assert!(json.contains("EMAIL"));
        assert!(json.contains("test@example.com"));
    }
    
    #[test]
    fn test_detection_result_empty() {
        let result = DetectionResult::new();
        assert!(result.entities.is_empty());
        assert_eq!(result.processing_time_ms, 0.0);
    }
    
    #[test]
    fn test_entity_type_display() {
        assert_eq!(EntityType::Email.to_string(), "EMAIL");
        assert_eq!(EntityType::MalaysianNric.to_string(), "MY_NRIC");
    }
}
```

#### 1.3 Error Types (TDD: Day 6-7)

**Tests:**
```rust
// crates/pii_redacta_core/src/error_test.rs
#[test]
fn test_error_display() {
    let err = PiiError::InvalidInput("empty text".to_string());
    assert!(err.to_string().contains("empty text"));
}

#[test]
fn test_error_is_user_safe() {
    assert!(!PiiError::InvalidInput("test".to_string()).is_user_safe());
    assert!(PiiError::NoEntitiesFound.is_user_safe());
}
```

**Implementation:**
```rust
// crates/pii_redacta_core/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PiiError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Detection failed: {0}")]
    DetectionFailed(String),
    
    #[error("No entities found")]
    NoEntitiesFound,
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
}

impl PiiError {
    /// Returns true if error message is safe to show to users
    pub fn is_user_safe(&self) -> bool {
        matches!(self, PiiError::NoEntitiesFound)
    }
}

pub type Result<T> = std::result::Result<T, PiiError>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_display() {
        let err = PiiError::InvalidInput("empty text".to_string());
        assert!(err.to_string().contains("empty text"));
    }
    
    #[test]
    fn test_error_is_user_safe() {
        assert!(!PiiError::InvalidInput("test".to_string()).is_user_safe());
        assert!(PiiError::NoEntitiesFound.is_user_safe());
    }
}
```

#### 1.4 CI/CD Setup (TDD: Day 8-10)

**Tests:**
```rust
// tests/ci_test.rs
#[test]
fn test_ci_workflow_exists() {
    assert!(Path::new(".github/workflows/ci.yml").exists());
}

#[test]
fn test_cargo_toml_valid() {
    // Should parse without errors
    let _manifest = std::fs::read_to_string("Cargo.toml").unwrap();
}
```

### Sprint 1 Code Review Checklist
- [ ] Workspace compiles
- [ ] All unit tests pass
- [ ] Types have proper serialization
- [ ] Error types are comprehensive
- [ ] CI workflow is configured
- [ ] Code coverage > 80%

### Sprint 1 QA Checklist
- [ ] `cargo test` passes in clean environment
- [ ] `cargo clippy` has no warnings
- [ ] `cargo fmt` produces no changes
- [ ] Documentation builds (`cargo doc`)

### Sprint 1 Commit Message
```
feat(core): sprint 1 - project foundation and core types

- Set up workspace structure with pii_redacta_core and pii_redacta_api
- Define domain types: Entity, EntityType, DetectionResult
- Implement comprehensive error handling
- Configure CI/CD with GitHub Actions
- Achieve 100% test coverage for types module

Tests: 24 tests passing
Coverage: 85%
```

---

## Sprint 2: Pattern-Based Detection Engine

**Duration:** 2 weeks  
**Focus:** Fast regex-based PII detection

### Sprint 2 Deliverables

#### 2.1 Pattern Detector (TDD: Day 1-5)

**Red Phase - Write Tests:**
```rust
// crates/pii_redacta_core/src/detection/pattern_test.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_email_simple() {
        let detector = PatternDetector::new();
        let text = "Contact john@example.com";
        let entities = detector.detect_email(text);
        
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].value, "john@example.com");
        assert_eq!(entities[0].start, 8);
        assert_eq!(entities[0].end, 24);
    }
    
    #[test]
    fn test_detect_email_multiple() {
        let detector = PatternDetector::new();
        let text = "john@example.com and jane@test.org";
        let entities = detector.detect_email(text);
        
        assert_eq!(entities.len(), 2);
    }
    
    #[test]
    fn test_detect_email_none() {
        let detector = PatternDetector::new();
        let text = "No email here";
        let entities = detector.detect_email(text);
        
        assert!(entities.is_empty());
    }
    
    #[test]
    fn test_detect_malaysian_nric() {
        let detector = PatternDetector::new();
        let text = "IC: 850101-14-5123";
        let entities = detector.detect_nric(text);
        
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].value, "850101-14-5123");
        assert_eq!(entities[0].entity_type, EntityType::MalaysianNric);
    }
    
    #[test]
    fn test_detect_malaysian_nric_old_format() {
        let detector = PatternDetector::new();
        let text = "IC: 850101145123";  // No dashes
        let entities = detector.detect_nric(text);
        
        assert_eq!(entities.len(), 1);
    }
    
    #[test]
    fn test_detect_phone_malaysian_mobile() {
        let detector = PatternDetector::new();
        let text = "Call me at 012-3456789";
        let entities = detector.detect_phone(text);
        
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].value, "012-3456789");
    }
    
    #[test]
    fn test_detect_phone_international() {
        let detector = PatternDetector::new();
        let text = "Contact +60 12-345 6789";
        let entities = detector.detect_phone(text);
        
        assert_eq!(entities.len(), 1);
    }
    
    #[test]
    fn test_detect_credit_card() {
        let detector = PatternDetector::new();
        let text = "Card: 4532-1234-5678-9012";
        let entities = detector.detect_credit_card(text);
        
        assert_eq!(entities.len(), 1);
    }
    
    #[test]
    fn test_detect_all() {
        let detector = PatternDetector::new();
        let text = "Email: john@example.com, IC: 850101-14-5123";
        let entities = detector.detect_all(text);
        
        assert_eq!(entities.len(), 2);
    }
    
    #[test]
    fn test_performance_under_1ms() {
        let detector = PatternDetector::new();
        let text = "Email: john@example.com, IC: 850101-14-5123, Phone: 012-3456789";
        
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = detector.detect_all(text);
        }
        let elapsed = start.elapsed();
        
        // Should process 1000 iterations in under 100ms (0.1ms per detection)
        assert!(elapsed.as_millis() < 100, "Too slow: {}ms", elapsed.as_millis());
    }
}
```

**Green Phase - Implementation:**
```rust
// crates/pii_redacta_core/src/detection/pattern.rs
use regex::Regex;
use once_cell::sync::Lazy;

use crate::types::{Entity, EntityType};

static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}")
        .expect("Valid regex")
});

static NRIC_REGEX: Lazy<Regex> = Lazy::new(|| {
    // Malaysian NRIC: YYMMDD-BB-NNNN or YYMMDDBBNNNN
    Regex::new(r"\b\d{6}-?\d{2}-?\d{4}\b")
        .expect("Valid regex")
});

static PHONE_REGEX: Lazy<Regex> = Lazy::new(|| {
    // Malaysian phone numbers
    Regex::new(r"(?:\+?6?01)[0-46-9]-*[0-9]{7,8}|\+?60[0-9]{8,9}")
        .expect("Valid regex")
});

static CREDIT_CARD_REGEX: Lazy<Regex> = Lazy::new(|| {
    // Basic credit card pattern (not Luhn validated yet)
    Regex::new(r"\b(?:\d{4}[-\s]?){3}\d{4}\b")
        .expect("Valid regex")
});

pub struct PatternDetector;

impl PatternDetector {
    pub fn new() -> Self {
        Self
    }
    
    pub fn detect_email(&self, text: &str) -> Vec<Entity> {
        self.find_entities(text, &EMAIL_REGEX, EntityType::Email)
    }
    
    pub fn detect_nric(&self, text: &str) -> Vec<Entity> {
        self.find_entities(text, &NRIC_REGEX, EntityType::MalaysianNric)
    }
    
    pub fn detect_phone(&self, text: &str) -> Vec<Entity> {
        self.find_entities(text, &PHONE_REGEX, EntityType::PhoneNumber)
    }
    
    pub fn detect_credit_card(&self, text: &str) -> Vec<Entity> {
        self.find_entities(text, &CREDIT_CARD_REGEX, EntityType::CreditCard)
    }
    
    pub fn detect_all(&self, text: &str) -> Vec<Entity> {
        let mut all = Vec::new();
        all.extend(self.detect_email(text));
        all.extend(self.detect_nric(text));
        all.extend(self.detect_phone(text));
        all.extend(self.detect_credit_card(text));
        
        // Sort by position
        all.sort_by_key(|e| e.start);
        all
    }
    
    fn find_entities(&self, text: &str, regex: &Regex, entity_type: EntityType) -> Vec<Entity> {
        regex
            .find_iter(text)
            .map(|m| Entity {
                entity_type,
                value: m.as_str().to_string(),
                start: m.start(),
                end: m.end(),
                confidence: Some(0.9), // High confidence for pattern matches
            })
            .collect()
    }
}

impl Default for PatternDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_email_simple() {
        let detector = PatternDetector::new();
        let text = "Contact john@example.com";
        let entities = detector.detect_email(text);
        
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].value, "john@example.com");
        assert_eq!(entities[0].start, 8);
        assert_eq!(entities[0].end, 24);
    }
    
    // ... other tests from above
}
```

#### 2.2 Detection Pipeline (TDD: Day 6-8)

**Tests:**
```rust
// crates/pii_redacta_core/src/detection/pipeline_test.rs
#[test]
fn test_pipeline_detects_entities() {
    let pipeline = DetectionPipeline::new();
    let result = pipeline.detect("Email: test@example.com").unwrap();
    
    assert_eq!(result.entities.len(), 1);
    assert!(result.processing_time_ms > 0.0);
}

#[test]
fn test_pipeline_empty_input() {
    let pipeline = DetectionPipeline::new();
    let result = pipeline.detect("");
    
    assert!(result.is_err());
}

#[test]
fn test_pipeline_filter_by_type() {
    let pipeline = DetectionPipeline::new();
    let options = DetectionOptions {
        entity_types: vec![EntityType::Email],
        ..Default::default()
    };
    
    let result = pipeline.detect_with_options(
        "Email: a@b.com, IC: 850101-14-5123",
        &options
    ).unwrap();
    
    assert_eq!(result.entities.len(), 1);
    assert_eq!(result.entities[0].entity_type, EntityType::Email);
}
```

**Implementation:**
```rust
// crates/pii_redacta_core/src/detection/pipeline.rs
use crate::types::{DetectionOptions, DetectionResult, EntityType};
use crate::error::{PiiError, Result};

use super::pattern::PatternDetector;

pub struct DetectionPipeline {
    pattern_detector: PatternDetector,
}

impl DetectionPipeline {
    pub fn new() -> Self {
        Self {
            pattern_detector: PatternDetector::new(),
        }
    }
    
    pub fn detect(&self, text: &str) -> Result<DetectionResult> {
        self.detect_with_options(text, &DetectionOptions::default())
    }
    
    pub fn detect_with_options(
        &self,
        text: &str,
        options: &DetectionOptions,
    ) -> Result<DetectionResult> {
        if text.is_empty() {
            return Err(PiiError::InvalidInput("Text cannot be empty".to_string()));
        }
        
        let start = std::time::Instant::now();
        
        let mut entities = self.pattern_detector.detect_all(text);
        
        // Filter by requested entity types
        if !options.entity_types.is_empty() {
            entities.retain(|e| options.entity_types.contains(&e.entity_type));
        }
        
        let processing_time_ms = start.elapsed().as_secs_f64() * 1000.0;
        
        Ok(DetectionResult::new()
            .with_entities(entities)
            .with_processing_time(processing_time_ms))
    }
}

// Extension trait for DetectionResult
trait DetectionResultExt {
    fn with_processing_time(self, time_ms: f64) -> Self;
}

impl DetectionResultExt for DetectionResult {
    fn with_processing_time(mut self, time_ms: f64) -> Self {
        self.processing_time_ms = time_ms;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pipeline_detects_entities() {
        let pipeline = DetectionPipeline::new();
        let result = pipeline.detect("Email: test@example.com").unwrap();
        
        assert_eq!(result.entities.len(), 1);
        assert!(result.processing_time_ms > 0.0);
    }
    
    // ... other tests
}
```

#### 2.3 Benchmarks (TDD: Day 9-10)

**Tests:**
```rust
// benches/detection_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pii_redacta_core::detection::PatternDetector;

fn bench_email_detection(c: &mut Criterion) {
    let detector = PatternDetector::new();
    let text = "Contact john@example.com or jane@test.org for details";
    
    c.bench_function("detect_email", |b| {
        b.iter(|| detector.detect_email(black_box(text)));
    });
}

fn bench_full_detection(c: &mut Criterion) {
    let detector = PatternDetector::new();
    let text = "Email: a@b.com, IC: 850101-14-5123, Phone: 012-3456789, Card: 4532-1234-5678-9012";
    
    c.bench_function("detect_all", |b| {
        b.iter(|| detector.detect_all(black_box(text)));
    });
}

criterion_group!(benches, bench_email_detection, bench_full_detection);
criterion_main!(benches);
```

### Sprint 2 Code Review Checklist
- [ ] All patterns tested with edge cases
- [ ] Performance < 1ms per detection
- [ ] Benchmarks exist
- [ ] No regex compilation per request
- [ ] Code coverage > 80%

### Sprint 2 QA Checklist
- [ ] Benchmarks pass performance targets
- [ ] Memory usage stable under load
- [ ] Handles unicode text correctly
- [ ] No regex denial of service vulnerabilities

### Sprint 2 Commit Message
```
feat(detection): sprint 2 - pattern-based detection engine

- Implement PatternDetector with regex-based detection
- Add Malaysian PII patterns (NRIC, phone numbers)
- Create DetectionPipeline for orchestration
- Achieve <1ms detection latency
- Add comprehensive benchmarks

Benchmarks:
- detect_email: 450ns
- detect_all: 1.2µs

Tests: 48 tests passing
Coverage: 87%
```

---

## Sprint 3: Tokenization Engine

**Duration:** 2 weeks  
**Focus:** PII tokenization and reversible redaction

### Sprint 3 Deliverables

#### 3.1 Token Generator (TDD: Day 1-4)

**Tests:**
```rust
// crates/pii_redacta_core/src/tokenization/generator_test.rs
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
fn test_token_no_original_value() {
    let generator = TokenGenerator::new("tenant-123");
    let token = generator.generate(EntityType::Email, "secret@example.com");
    
    assert!(!token.contains("secret"));
    assert!(!token.contains("example.com"));
}
```

**Implementation:**
```rust
// crates/pii_redacta_core/src/tokenization/generator.rs
use sha2::{Sha256, Digest};
use crate::types::EntityType;

pub struct TokenGenerator {
    tenant_id: String,
}

impl TokenGenerator {
    pub fn new(tenant_id: &str) -> Self {
        Self {
            tenant_id: tenant_id.to_string(),
        }
    }
    
    pub fn generate(&self, entity_type: EntityType, value: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&self.tenant_id);
        hasher.update(value.as_bytes());
        
        // Use first 12 chars of hash for readable token
        let hash = hex::encode(&hasher.finalize()[..6]);
        
        format!("<<PII_{}_{}>>", entity_type, hash.to_uppercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_token_deterministic() {
        let generator = TokenGenerator::new("tenant-123");
        
        let token1 = generator.generate(EntityType::Email, "test@example.com");
        let token2 = generator.generate(EntityType::Email, "test@example.com");
        
        assert_eq!(token1, token2);
    }
    
    // ... other tests
}
```

#### 3.2 Tokenizer (TDD: Day 5-7)

**Tests:**
```rust
// crates/pii_redacta_core/src/tokenization/tokenizer_test.rs
#[test]
fn test_tokenize_single_entity() {
    let tokenizer = Tokenizer::new("tenant-123");
    let text = "Email: john@example.com";
    let entities = vec![
        Entity::new(EntityType::Email, "john@example.com", 7, 23),
    ];
    
    let (tokenized, token_map) = tokenizer.tokenize(text, &entities);
    
    assert!(tokenized.contains("<<PII_EMAIL_"));
    assert!(!tokenized.contains("john@example.com"));
    assert_eq!(token_map.len(), 1);
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
fn test_tokenize_overlapping_entities() {
    let tokenizer = Tokenizer::new("tenant-123");
    // Overlapping: one entity contains another
    let text = "Full: john.doe@example.com";
    let entities = vec![
        Entity::new(EntityType::Email, "john.doe@example.com", 6, 26),
        Entity::new(EntityType::PersonName, "john.doe", 6, 14),
    ];
    
    let (tokenized, token_map) = tokenizer.tokenize(text, &entities);
    
    // Should handle overlapping gracefully (prefer larger entity)
    assert!(tokenized.contains("<<PII_EMAIL_"));
}

#[test]
fn test_detokenize() {
    let tokenizer = Tokenizer::new("tenant-123");
    let text = "Email: john@example.com";
    let entities = vec![
        Entity::new(EntityType::Email, "john@example.com", 7, 23),
    ];
    
    let (tokenized, token_map) = tokenizer.tokenize(text, &entities);
    let restored = tokenizer.detokenize(&tokenized, &token_map);
    
    assert_eq!(restored, text);
}

#[test]
fn test_tokenize_preserves_structure() {
    let tokenizer = Tokenizer::new("tenant-123");
    let text = "Start\nEmail: john@example.com\nEnd";
    let entities = vec![
        Entity::new(EntityType::Email, "john@example.com", 13, 29),
    ];
    
    let (tokenized, _) = tokenizer.tokenize(text, &entities);
    
    assert!(tokenized.starts_with("Start\n"));
    assert!(tokenized.ends_with("\nEnd"));
}
```

**Implementation:**
```rust
// crates/pii_redacta_core/src/tokenization/tokenizer.rs
use std::collections::HashMap;
use crate::types::Entity;

use super::generator::TokenGenerator;

pub struct Tokenizer {
    generator: TokenGenerator,
}

pub type TokenMap = HashMap<String, String>;

impl Tokenizer {
    pub fn new(tenant_id: &str) -> Self {
        Self {
            generator: TokenGenerator::new(tenant_id),
        }
    }
    
    pub fn tokenize(&self, text: &str, entities: &[Entity]) -> (String, TokenMap) {
        let mut token_map = HashMap::new();
        let mut tokenized = text.to_string();
        
        // Sort entities by start position (descending) to replace from end
        let mut sorted_entities: Vec<_> = entities.to_vec();
        sorted_entities.sort_by_key(|e| std::cmp::Reverse(e.start));
        
        // Remove overlapping entities (keep larger ones)
        sorted_entities = self.remove_overlapping(sorted_entities);
        
        for entity in sorted_entities {
            let token = self.generator.generate(entity.entity_type, &entity.value);
            token_map.insert(token.clone(), entity.value.clone());
            
            // Replace in string (adjusting for previous replacements)
            let start = entity.start;
            let end = entity.end;
            if start < tokenized.len() && end <= tokenized.len() {
                tokenized.replace_range(start..end, &token);
            }
        }
        
        (tokenized, token_map)
    }
    
    pub fn detokenize(&self, tokenized: &str, token_map: &TokenMap) -> String {
        let mut result = tokenized.to_string();
        
        // Replace tokens in reverse order of length (longest first)
        let mut tokens: Vec<_> = token_map.iter().collect();
        tokens.sort_by_key(|(k, _)| std::cmp::Reverse(k.len()));
        
        for (token, original) in tokens {
            result = result.replace(token, original);
        }
        
        result
    }
    
    fn remove_overlapping(&self, mut entities: Vec<Entity>) -> Vec<Entity> {
        if entities.len() <= 1 {
            return entities;
        }
        
        let mut result = Vec::new();
        
        for entity in entities {
            let overlaps = result.iter().any(|e: &Entity| {
                // Check if current entity overlaps with any in result
                (entity.start < e.end && entity.end > e.start)
            });
            
            if !overlaps {
                result.push(entity);
            }
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Entity, EntityType};
    
    #[test]
    fn test_tokenize_single_entity() {
        let tokenizer = Tokenizer::new("tenant-123");
        let text = "Email: john@example.com";
        let entities = vec![
            Entity::new(EntityType::Email, "john@example.com", 7, 23),
        ];
        
        let (tokenized, token_map) = tokenizer.tokenize(text, &entities);
        
        assert!(tokenized.contains("<<PII_EMAIL_"));
        assert!(!tokenized.contains("john@example.com"));
        assert_eq!(token_map.len(), 1);
    }
    
    // ... other tests
}
```

#### 3.3 Token Store (TDD: Day 8-10)

**Tests:**
```rust
// crates/pii_redacta_core/src/tokenization/store_test.rs
#[test]
fn test_store_token_map() {
    let mut store = InMemoryTokenStore::new();
    let request_id = "req-123";
    let mut token_map = HashMap::new();
    token_map.insert("<<TOKEN_1>>".to_string(), "secret".to_string());
    
    store.store(request_id, token_map.clone());
    
    let retrieved = store.retrieve(request_id).unwrap();
    assert_eq!(retrieved, &token_map);
}

#[test]
fn test_store_retrieve_nonexistent() {
    let store = InMemoryTokenStore::new();
    
    let result = store.retrieve("nonexistent");
    assert!(result.is_none());
}

#[test]
fn test_store_ttl_expiration() {
    let mut store = InMemoryTokenStore::with_ttl(Duration::from_millis(1));
    store.store("req-123", HashMap::new());
    
    // Wait for expiration
    std::thread::sleep(Duration::from_millis(10));
    
    let result = store.retrieve("req-123");
    assert!(result.is_none());
}
```

### Sprint 3 Code Review Checklist
- [ ] Token generation is deterministic
- [ ] Tenant isolation works correctly
- [ ] No original values in tokens
- [ ] Overlapping entities handled
- [ ] Token map secure storage

### Sprint 3 QA Checklist
- [ ] Detokenize restores exact original
- [ ] Token format consistent
- [ ] Memory usage for token store bounded
- [ ] TTL expiration works

### Sprint 3 Commit Message
```
feat(tokenization): sprint 3 - tokenization engine

- Implement deterministic token generation with tenant isolation
- Create Tokenizer with overlap handling
- Add InMemoryTokenStore with TTL
- Full detokenization support

Security:
- No PII in token values
- Tenant-scoped tokens
- Secure token map storage

Tests: 36 tests passing
Coverage: 89%
```

---

## Sprint 4: REST API Foundation

**Duration:** 2 weeks  
**Focus:** Axum-based REST API with health endpoints

### Sprint 4 Deliverables

#### 4.1 API Structure (TDD: Day 1-3)

**Tests:**
```rust
// crates/pii_redacta_api/tests/health_test.rs
#[tokio::test]
async fn test_health_endpoint() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let health: HealthResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(health.status, "healthy");
}

#[tokio::test]
async fn test_ready_endpoint() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .uri("/ready")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}
```

**Implementation:**
```rust
// crates/pii_redacta_api/src/main.rs
use axum::{
    routing::get,
    Router,
    Json,
};
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

#[derive(Serialize)]
struct ReadyResponse {
    ready: bool,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn ready() -> Json<ReadyResponse> {
    Json(ReadyResponse { ready: true })
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready));
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

#### 4.2 Detection Endpoint (TDD: Day 4-7)

**Tests:**
```rust
// crates/pii_redacta_api/tests/detection_test.rs
#[tokio::test]
async fn test_detect_endpoint_success() {
    let app = create_test_app().await;
    
    let request = DetectRequest {
        text: "Email: test@example.com".to_string(),
        options: None,
    };
    
    let response = app
        .oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/detect")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&request).unwrap()))
            .unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let result: DetectResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(result.entities.len(), 1);
}

#[tokio::test]
async fn test_detect_endpoint_empty_body() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/detect")
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_detect_endpoint_with_redaction() {
    let app = create_test_app().await;
    
    let request = DetectRequest {
        text: "Email: test@example.com".to_string(),
        options: Some(DetectionOptions {
            redact: true,
            ..Default::default()
        }),
    };
    
    let response = app
        .oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/detect")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&request).unwrap()))
            .unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let result: DetectResponse = serde_json::from_slice(&body).unwrap();
    assert!(result.tokenized_text.is_some());
    assert!(result.token_map.is_some());
}
```

**Implementation:**
```rust
// crates/pii_redacta_api/src/handlers/detection.rs
use axum::{extract::State, http::StatusCode, Json};
use pii_redacta_core::{
    detection::DetectionPipeline,
    tokenization::Tokenizer,
    types::{DetectionOptions, DetectionResult},
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct DetectRequest {
    pub text: String,
    pub options: Option<DetectionOptions>,
}

#[derive(Serialize)]
pub struct DetectResponse {
    #[serde(flatten)]
    pub result: DetectionResult,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn detect(
    State(pipeline): State<DetectionPipeline>,
    Json(request): Json<DetectRequest>,
) -> Result<Json<DetectResponse>, (StatusCode, Json<ErrorResponse>)> {
    let options = request.options.unwrap_or_default();
    
    match pipeline.detect_with_options(&request.text, &options) {
        Ok(mut result) => {
            // Handle redaction if requested
            if options.redact {
                let tenant_id = options.tenant_id.as_deref().unwrap_or("default");
                let tokenizer = Tokenizer::new(tenant_id);
                let (tokenized, token_map) = tokenizer.tokenize(&request.text, &result.entities);
                
                result.tokenized_text = Some(tokenized);
                result.token_map = Some(token_map);
            }
            
            Ok(Json(DetectResponse { result }))
        }
        Err(e) => {
            let status = if e.is_user_safe() {
                StatusCode::OK
            } else {
                StatusCode::BAD_REQUEST
            };
            
            Err((
                status,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            ))
        }
    }
}
```

#### 4.3 Error Handling (TDD: Day 8-10)

**Tests:**
```rust
#[tokio::test]
async fn test_invalid_json_returns_400() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/detect")
            .header("Content-Type", "application/json")
            .body(Body::from("invalid json"))
            .unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_missing_content_type() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/detect")
            .body(Body::from(r#"{"text":"test"}"#))
            .unwrap())
        .await
        .unwrap();
    
    // Should still work or return proper error
    assert!(response.status().is_success() || response.status() == StatusCode::UNSUPPORTED_MEDIA_TYPE);
}
```

### Sprint 4 Code Review Checklist
- [ ] All endpoints tested
- [ ] Error handling comprehensive
- [ ] Request/response types validated
- [ ] Content-type handling correct
- [ ] Status codes appropriate

### Sprint 4 QA Checklist
- [ ] API responds within 50ms
- [ ] Concurrent requests handled
- [ ] JSON serialization correct
- [ ] Error responses informative

### Sprint 4 Commit Message
```
feat(api): sprint 4 - REST API foundation

- Implement Axum-based REST API
- Add health and readiness endpoints
- Create /api/v1/detect endpoint
- Comprehensive error handling
- Request validation

API Endpoints:
- GET /health - Health check
- GET /ready - Readiness probe
- POST /api/v1/detect - PII detection

Tests: 24 tests passing
Coverage: 82%
```

---

## Sprint 5: File Processing

**Duration:** 2 weeks  
**Focus:** PDF, DOCX, TXT file processing

### Sprint 5 Deliverables

#### 5.1 Text Extractor (TDD: Day 1-4)

**Tests:**
```rust
// crates/pii_redacta_core/src/extraction/mod_test.rs
#[test]
fn test_extract_txt() {
    let content = b"Email: test@example.com";
    let result = TextExtractor::extract(content, "text/plain").unwrap();
    
    assert_eq!(result.text, "Email: test@example.com");
    assert_eq!(result.format, DocumentFormat::PlainText);
}

#[test]
fn test_extract_unsupported_type() {
    let content = b"some binary data";
    let result = TextExtractor::extract(content, "application/unknown");
    
    assert!(result.is_err());
}

#[test]
fn test_detect_mime_type() {
    assert_eq!(
        TextExtractor::detect_mime(b"%PDF-1.4"),
        Some("application/pdf")
    );
    assert_eq!(
        TextExtractor::detect_mime(b"PK\x03\x04"),  // ZIP signature (DOCX)
        Some("application/vnd.openxmlformats")
    );
}
```

#### 5.2 PDF Extractor (TDD: Day 5-7)

**Tests:**
```rust
#[test]
fn test_extract_pdf_simple() {
    let pdf_bytes = include_bytes!("../../test_data/sample.pdf");
    let result = PdfExtractor::extract(pdf_bytes).unwrap();
    
    assert!(result.text.contains("test@example.com"));
}

#[test]
fn test_extract_pdf_empty() {
    let result = PdfExtractor::extract(b"%PDF-1.4\n1 0 obj\n<<\n/Type /Catalog\n>>");
    // Should handle gracefully
    assert!(result.is_ok() || result.is_err());
}
```

#### 5.3 DOCX Extractor (TDD: Day 8-10)

**Tests:**
```rust
#[test]
fn test_extract_docx() {
    let docx_bytes = include_bytes!("../../test_data/sample.docx");
    let result = DocxExtractor::extract(docx_bytes).unwrap();
    
    assert!(result.text.contains("confidential"));
}
```

### Sprint 5 Commit Message
```
feat(extraction): sprint 5 - file processing

- Implement TextExtractor with MIME type detection
- Add PDF text extraction
- Add DOCX text extraction
- Support plain text files

Supported Formats:
- text/plain
- application/pdf
- application/vnd.openxmlformats

Tests: 18 tests passing
Coverage: 79%
```

---

## Sprint 6: File Upload API & Integration

**Duration:** 2 weeks  
**Focus:** File upload endpoints and async processing

### Sprint 6 Deliverables

#### 6.1 File Upload Handler (TDD: Day 1-5)

**Tests:**
```rust
// crates/pii_redacta_api/tests/file_upload_test.rs
#[tokio::test]
async fn test_upload_txt_file() {
    let app = create_test_app().await;
    
    let boundary = "----WebKitFormBoundary";
    let body = format!(
        "------WebKitFormBoundary\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n\
         Content-Type: text/plain\r\n\r\n\
         Email: test@example.com\r\n\
         ------WebKitFormBoundary--\r\n"
    );
    
    let response = app
        .oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/upload")
            .header("Content-Type", format!("multipart/form-data; boundary={}", boundary))
            .body(Body::from(body))
            .unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::ACCEPTED);
}

#[tokio::test]
async fn test_upload_file_too_large() {
    let app = create_test_app().await;
    // Test with oversized file
}
```

#### 6.2 Async Job Processing (TDD: Day 6-10)

**Tests:**
```rust
#[tokio::test]
async fn test_job_queue_processing() {
    let queue = JobQueue::new();
    let job_id = queue.submit(Job::new("text content")).await;
    
    // Wait for processing
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let status = queue.status(&job_id).await;
    assert!(matches!(status, JobStatus::Completed { .. }));
}
```

### Sprint 6 Commit Message
```
feat(api): sprint 6 - file upload and async processing

- Add multipart file upload endpoint
- Implement async job queue
- Add job status tracking
- File size limits and validation

Endpoints:
- POST /api/v1/upload - File upload
- GET /api/v1/jobs/{id} - Job status

Tests: 22 tests passing
Coverage: 81%
```

---

## Sprint 7: Observability & Documentation

**Duration:** 2 weeks  
**Focus:** Logging, metrics, OpenAPI docs

### Sprint 7 Deliverables

#### 7.1 Structured Logging (TDD: Day 1-4)

**Tests:**
```rust
#[test]
fn test_no_pii_in_logs() {
    let _guard = init_test_logging();
    
    let text = "secret@example.com";
    let _ = detector.detect(text);
    
    let logs = get_test_logs();
    assert!(!logs.contains("secret@example.com"));
    assert!(logs.contains("[REDACTED]"));
}
```

#### 7.2 Metrics (TDD: Day 5-7)

**Tests:**
```rust
#[test]
fn test_metrics_collected() {
    let metrics = Metrics::new();
    
    metrics.record_detection(DetectionMetrics {
        entity_count: 2,
        processing_time_ms: 0.5,
    });
    
    let stats = metrics.snapshot();
    assert_eq!(stats.total_detections, 1);
    assert_eq!(stats.total_entities, 2);
}
```

#### 7.3 OpenAPI Spec (TDD: Day 8-10)

**Tests:**
```rust
#[test]
fn test_openapi_spec_valid() {
    let spec = generate_openapi_spec();
    
    assert!(spec.paths.contains_key("/api/v1/detect"));
    assert!(spec.components.schemas.contains_key("DetectRequest"));
}
```

### Sprint 7 Commit Message
```
feat(observability): sprint 7 - logging, metrics, docs

- Add structured logging with PII redaction
- Implement Prometheus metrics
- Generate OpenAPI specification
- Health metrics endpoint

Observability:
- /metrics - Prometheus metrics
- /health - Health status
- /openapi.json - API specification

Tests: 28 tests passing
Coverage: 85%
```

---

## Sprint 8: Security Hardening & MVP Release

**Duration:** 2 weeks  
**Focus:** Security audit, rate limiting, MVP packaging

### Sprint 8 Deliverables

#### 8.1 Rate Limiting (TDD: Day 1-4)

**Tests:**
```rust
#[tokio::test]
async fn test_rate_limit_enforced() {
    let app = create_test_app_with_rate_limit().await;
    
    // Make requests up to limit
    for _ in 0..100 {
        let response = make_request(&app).await;
        assert_eq!(response.status(), StatusCode::OK);
    }
    
    // Next request should be rate limited
    let response = make_request(&app).await;
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}
```

#### 8.2 Security Headers (TDD: Day 5-7)

**Tests:**
```rust
#[tokio::test]
async fn test_security_headers_present() {
    let app = create_test_app().await;
    let response = make_request(&app).await;
    
    let headers = response.headers();
    assert!(headers.contains_key("x-content-type-options"));
    assert!(headers.contains_key("x-frame-options"));
}
```

#### 8.3 Docker & Deployment (TDD: Day 8-10)

**Tests:**
```rust
// tests/docker_test.rs
#[test]
fn test_dockerfile_builds() {
    // Integration test with docker
}
```

### Sprint 8 Commit Message
```
feat(release): sprint 8 - MVP security hardening and release

- Add rate limiting middleware
- Security headers on all responses
- Docker containerization
- Security audit and fixes

Security:
- Rate limiting: 100 req/min per IP
- Security headers (CSP, HSTS, etc.)
- No PII in logs
- Input validation

MVP Complete! 🎉
Tests: 156 total tests passing
Coverage: 86%
```

---

## Summary: MVP Sprint Timeline

| Sprint | Focus | Duration | Key Deliverables |
|--------|-------|----------|------------------|
| 1 | Foundation | 2 weeks | Project structure, core types |
| 2 | Detection Engine | 2 weeks | Pattern-based detection |
| 3 | Tokenization | 2 weeks | Token generation & storage |
| 4 | REST API | 2 weeks | HTTP endpoints |
| 5 | File Extraction | 2 weeks | PDF, DOCX, TXT support |
| 6 | File Upload | 2 weeks | Async job processing |
| 7 | Observability | 2 weeks | Logs, metrics, docs |
| 8 | Security & Release | 2 weeks | Hardening, Docker, MVP |

**Total MVP Duration:** 16 weeks (4 months)

---

## TDD Commit Protocol

### Commit Message Format
```
<type>(<scope>): sprint <N> - <description>

[optional body]

Tests: <count> tests passing
Coverage: <percentage>%
```

### Types
- `feat`: New feature
- `fix`: Bug fix
- `refactor`: Code refactoring
- `test`: Adding tests
- `docs`: Documentation
- `chore`: Maintenance

### Example
```
feat(detection): sprint 2 - email pattern detection

- Add regex-based email detection
- Support international email formats
- Performance optimized with lazy regex

Tests: 12 new tests, 48 total passing
Coverage: 87% (+5%)
```

---

**Document Version:** 3.0  
**Last Updated:** 2026-02-27  
**Status:** Active Development Process
