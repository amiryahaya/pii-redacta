# Test Driven Development (TDD) Guide for PII Redacta

## Overview

This project follows **Test Driven Development (TDD)** methodology:

```
┌─────────┐    ┌─────────┐    ┌─────────┐
│  Write  │───▶│  Write  │───▶│ Refactor│
│  Test   │    │  Code   │    │         │
│ (Red)   │    │(Green)  │    │(Blue)   │
└─────────┘    └─────────┘    └─────────┘
     ▲                              │
     └──────────────────────────────┘
           Repeat Cycle
```

**The TDD Cycle:**
1. **Red:** Write a failing test
2. **Green:** Write minimal code to pass the test
3. **Blue:** Refactor while keeping tests green
4. **Repeat**

---

## TDD Principles for PII Redacta

### 1. Test-First Development

**NEVER** write production code without a failing test first.

```rust
// ❌ DON'T: Write implementation first
pub fn detect_email(text: &str) -> Vec<Email> {
    // Implementation...
}

// ✅ DO: Write test first
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_email_simple() {
        let text = "Contact john@example.com";
        let emails = detect_email(text);
        assert_eq!(emails.len(), 1);
        assert_eq!(emails[0].value, "john@example.com");
    }
}

// Then write minimal implementation
pub fn detect_email(text: &str) -> Vec<Email> {
    // Minimal code to pass test
}
```

### 2. Three Laws of TDD

1. **You can't write production code** until you have a failing unit test
2. **You can't write more of a unit test** than is sufficient to fail (compile failures are failures)
3. **You can't write more production code** than is sufficient to pass the currently failing test

### 3. Test Pyramid

```
         /\
        /  \
       / E2E\        ~5%  (Integration tests)
      /--------\
     /  Integration\   ~15% (API tests)
    /----------------\
   /    Unit Tests     \  ~80% (Core logic)
  /----------------------\
```

---

## Project Structure for TDD

```
crates/
├── pii_redacta_core/
│   └── src/
│       ├── lib.rs
│       ├── detection/
│       │   ├── mod.rs
│       │   ├── patterns.rs
│       │   └── patterns_test.rs      # Tests alongside code
│       └── tokenization/
│           ├── mod.rs
│           └── tokenization_test.rs
│   └── tests/                         # Integration tests
│       └── detection_integration.rs
│
└── pii_redacta_api/
    └── src/
        ├── main.rs
        ├── handlers/
        │   ├── detection.rs
│       │   └── detection_test.rs
```

---

## TDD Workflow by Example

### Example 1: Pattern Detection (Email)

**Step 1: Write Failing Test (Red)**

```rust
// crates/pii_redacta_core/src/detection/patterns_test.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_simple_email() {
        let text = "Contact john@example.com for details";
        let entities = detect_email(text);
        
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].value, "john@example.com");
        assert_eq!(entities[0].start, 8);
        assert_eq!(entities[0].end, 24);
    }
    
    #[test]
    fn test_detect_no_email() {
        let text = "This text has no email address";
        let entities = detect_email(text);
        
        assert!(entities.is_empty());
    }
    
    #[test]
    fn test_detect_multiple_emails() {
        let text = "john@example.com and jane@test.org";
        let entities = detect_email(text);
        
        assert_eq!(entities.len(), 2);
    }
}
```

Run test (should fail):
```bash
cargo test test_detect_simple_email
# FAIL: function `detect_email` not found
```

**Step 2: Write Minimal Code (Green)**

```rust
// crates/pii_redacta_core/src/detection/patterns.rs
use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
    pub value: String,
    pub start: usize,
    pub end: usize,
}

pub fn detect_email(text: &str) -> Vec<Entity> {
    // Minimal implementation to pass tests
    let email_regex = Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();
    
    email_regex
        .find_iter(text)
        .map(|m| Entity {
            value: m.as_str().to_string(),
            start: m.start(),
            end: m.end(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_simple_email() {
        let text = "Contact john@example.com for details";
        let entities = detect_email(text);
        
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].value, "john@example.com");
        assert_eq!(entities[0].start, 8);
        assert_eq!(entities[0].end, 24);
    }
    
    #[test]
    fn test_detect_no_email() {
        let text = "This text has no email address";
        let entities = detect_email(text);
        
        assert!(entities.is_empty());
    }
    
    #[test]
    fn test_detect_multiple_emails() {
        let text = "john@example.com and jane@test.org";
        let entities = detect_email(text);
        
        assert_eq!(entities.len(), 2);
    }
}
```

Run tests (should pass):
```bash
cargo test test_detect
# PASS: All tests pass
```

**Step 3: Refactor (Blue)**

```rust
// Refactored with better error handling and performance
use regex::Regex;
use once_cell::sync::Lazy;

// Compile regex once
static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}")
        .expect("Valid regex")
});

pub fn detect_email(text: &str) -> Vec<Entity> {
    EMAIL_REGEX
        .find_iter(text)
        .map(|m| Entity {
            value: m.as_str().to_string(),
            start: m.start(),
            end: m.end(),
        })
        .collect()
}
```

Run tests again:
```bash
cargo test test_detect
# PASS: All tests still pass after refactor
```

---

### Example 2: Tokenization

**Step 1: Write Failing Test**

```rust
// crates/pii_redacta_core/src/tokenization/tokenizer_test.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tokenize_simple() {
        let text = "Email: john@example.com";
        let entities = vec![Entity {
            value: "john@example.com".to_string(),
            entity_type: EntityType::Email,
            start: 7,
            end: 23,
        }];
        
        let tokenizer = Tokenizer::new("tenant-123");
        let (tokenized, token_map) = tokenizer.tokenize(text, &entities);
        
        assert!(tokenized.contains("<<PII_EMAIL_"));
        assert!(!tokenized.contains("john@example.com"));
        assert_eq!(token_map.len(), 1);
    }
    
    #[test]
    fn test_deterministic_tokenization() {
        // Same value should produce same token
        let text1 = "john@example.com";
        let text2 = "john@example.com";
        
        let tokenizer = Tokenizer::new("tenant-123");
        let (tok1, _) = tokenizer.tokenize(text1, &create_email_entity(text1));
        let (tok2, _) = tokenizer.tokenize(text2, &create_email_entity(text2));
        
        assert_eq!(tok1, tok2);
    }
}
```

**Step 2: Minimal Implementation**

```rust
// crates/pii_redacta_core/src/tokenization/tokenizer.rs
use sha2::{Sha256, Digest};

pub struct Tokenizer {
    tenant_id: String,
}

impl Tokenizer {
    pub fn new(tenant_id: &str) -> Self {
        Self {
            tenant_id: tenant_id.to_string(),
        }
    }
    
    pub fn tokenize(&self, text: &str, entities: &[Entity]) -> (String, HashMap<String, String>) {
        let mut tokenized = text.to_string();
        let mut token_map = HashMap::new();
        
        // Process entities in reverse order to preserve positions
        for entity in entities.iter().rev() {
            let token = self.generate_token(&entity.entity_type, &entity.value);
            token_map.insert(token.clone(), entity.value.clone());
            
            tokenized.replace_range(entity.start..entity.end, &token);
        }
        
        (tokenized, token_map)
    }
    
    fn generate_token(&self, entity_type: &EntityType, value: &str) -> String {
        // Deterministic hash
        let mut hasher = Sha256::new();
        hasher.update(&self.tenant_id);
        hasher.update(value);
        let hash = hex::encode(&hasher.finalize()[..8]);
        
        format!("<<PII_{}_{}>>", entity_type, hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // ... tests from above
}
```

---

## Testing Strategy

### 1. Unit Tests (80%)

**Location:** Inline with source code (`mod tests`)

**Characteristics:**
- Fast (< 10ms per test)
- Isolated (no DB, no network)
- Deterministic
- Tests single function/method

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pattern_email() {
        // Fast, isolated, deterministic
    }
}
```

### 2. Integration Tests (15%)

**Location:** `tests/` directory

**Characteristics:**
- Test component interaction
- May use test database
- Test API endpoints

```rust
// tests/detection_integration.rs
#[tokio::test]
async fn test_detection_api() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/detect")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"text": "test@example.com"}"#))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}
```

### 3. E2E Tests (5%)

**Location:** `tests/e2e/`

**Characteristics:**
- Full system testing
- Real (or realistic) dependencies
- Critical user journeys

```rust
// tests/e2e/file_processing_test.rs
#[tokio::test]
async fn test_full_file_processing_flow() {
    // Upload file -> Process -> Download redacted
}
```

---

## Test Categories

### 1. Happy Path Tests

```rust
#[test]
fn test_detect_email_success() {
    let text = "Valid email: user@example.com";
    let result = detect_email(text);
    assert_eq!(result.len(), 1);
}
```

### 2. Edge Case Tests

```rust
#[test]
fn test_detect_email_unicode() {
    let text = "用户@例子.测试";  // Unicode email
    let result = detect_email(text);
    // Should handle or gracefully skip
}

#[test]
fn test_detect_email_empty() {
    let text = "";
    let result = detect_email(text);
    assert!(result.is_empty());
}

#[test]
fn test_detect_email_very_long() {
    let text = "a".repeat(1_000_000);
    let result = detect_email(&text);
    // Should not panic, handle gracefully
}
```

### 3. Security Tests

```rust
#[test]
fn test_no_pii_in_logs() {
    let text = "secret@example.com";
    
    // Process with logging
    let _ = detector.detect(text);
    
    // Verify logs don't contain PII
    let logs = get_test_logs();
    assert!(!logs.contains("secret@example.com"));
}

#[test]
fn test_secure_memory_zeroization() {
    let arena = SecureArena::new(1024).unwrap();
    arena.store("sensitive data").unwrap();
    arena.zeroize();
    
    // Verify buffer is zeroed
    assert!(arena.buffer.iter().all(|&b| b == 0));
}
```

### 4. Performance Tests

```rust
// benches/detection_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_email_detection(c: &mut Criterion) {
    let text = "Contact john@example.com or jane@test.org";
    
    c.bench_function("detect_email", |b| {
        b.iter(|| detect_email(black_box(text)));
    });
}

criterion_group!(benches, bench_email_detection);
criterion_main!(benches);
```

---

## CI/CD Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
        components: rustfmt, clippy
    
    - name: Cache cargo
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Check formatting
      run: cargo fmt -- --check
    
    - name: Run clippy
      run: cargo clippy -- -D warnings
    
    - name: Run tests
      run: cargo test --all-features
      
    - name: Run doc tests
      run: cargo test --doc
      
    - name: Run coverage
      run: |
        cargo install cargo-tarpaulin
        cargo tarpaulin --out Xml
        
    - name: Upload coverage
      uses: codecov/codecov-action@v3
      with:
        file: ./cobertura.xml
```

### Pre-commit Hooks

```bash
# .git/hooks/pre-commit
#!/bin/bash

# Format check
cargo fmt -- --check
if [ $? -ne 0 ]; then
    echo "Formatting check failed. Run 'cargo fmt' to fix."
    exit 1
fi

# Clippy check
cargo clippy -- -D warnings
if [ $? -ne 0 ]; then
    echo "Clippy check failed."
    exit 1
fi

# Run tests
cargo test
if [ $? -ne 0 ]; then
    echo "Tests failed."
    exit 1
fi

echo "All checks passed!"
```

---

## TDD Best Practices

### 1. Test Naming

```rust
// ✅ Good: Descriptive, follows pattern
#[test]
fn test_detect_email_with_subdomain() { }

#[test]
fn test_tokenize_returns_deterministic_tokens() { }

#[test]
fn test_secure_arena_zeroizes_on_drop() { }

// ❌ Bad: Vague names
#[test]
fn test1() { }

#[test]
fn email_test() { }
```

### 2. Arrange-Act-Assert

```rust
#[test]
fn test_detect_malaysian_nric() {
    // Arrange
    let text = "IC: 850101-14-5123";
    let expected = "850101-14-5123";
    
    // Act
    let result = detect_nric(text);
    
    // Assert
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].value, expected);
}
```

### 3. One Concept Per Test

```rust
// ✅ Good: Single concept
#[test]
fn test_detect_email_finds_single_email() { }

#[test]
fn test_detect_email_finds_multiple_emails() { }

#[test]
fn test_detect_email_returns_empty_for_no_match() { }

// ❌ Bad: Multiple concepts
#[test]
fn test_email_detection() {
    // Tests single, multiple, empty all in one
}
```

### 4. Use Given-When-Then for BDD Style

```rust
#[test]
fn test_file_processing() {
    // Given: A PDF file with PII
    let file = load_test_file("sample_with_pii.pdf");
    
    // When: Processing the file
    let result = processor.process(file).await;
    
    // Then: PII should be redacted
    assert!(result.redacted_text.contains("<<PII_"));
    assert!(!result.redacted_text.contains("john@example.com"));
}
```

---

## Common TDD Mistakes

### 1. Testing Implementation Details

```rust
// ❌ Bad: Testing internal state
#[test]
fn test_internal_counter() {
    let mut detector = Detector::new();
    detector.process("test");
    assert_eq!(detector.internal_counter, 1);  // Implementation detail!
}

// ✅ Good: Testing behavior
#[test]
fn test_detects_all_entities() {
    let result = detector.process("john@example.com and 123-45-6789");
    assert_eq!(result.entities.len(), 2);
}
```

### 2. Brittle Tests

```rust
// ❌ Bad: Testing exact error message
#[test]
fn test_error_message() {
    let err = process("");
    assert_eq!(err.to_string(), "Exact error message that might change");
}

// ✅ Good: Testing error type
#[test]
fn test_error_empty_input() {
    let err = process("");
    assert!(matches!(err, Error::EmptyInput));
}
```

### 3. Slow Tests

```rust
// ❌ Bad: Hitting real services
#[test]
fn test_with_real_api() {
    let result = call_production_api();  // Slow, unreliable
}

// ✅ Good: Mock external services
#[test]
fn test_with_mock() {
    let mock = MockApi::new().return_ok();
    let result = process_with_mock(mock);
}
```

---

## Testing Tools

### Essential Crates

```toml
[dev-dependencies]
# Testing framework
tokio-test = "0.4"

# Mocking
mockall = "0.12"

# Assertions
assert_matches = "1.5"
pretty_assertions = "1.4"

# Property-based testing
proptest = "1.4"

# Test fixtures
tempfile = "3.9"

# HTTP testing (for API)
reqwest = { version = "0.11", features = ["json"] }
wiremock = "0.6"

# Coverage
cargo-tarpaulin = "0.27"
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_detect_email

# Run with output
cargo test -- --nocapture

# Run ignored tests
cargo test -- --ignored

# Run benchmarks
cargo bench

# Generate coverage
cargo tarpaulin --out Html

# Check without running
cargo check

# Clippy lints
cargo clippy

# Format code
cargo fmt
```

---

## TDD Checklist

Before committing code:

- [ ] All tests pass (`cargo test`)
- [ ] New code has tests
- [ ] Tests fail before implementation (Red)
- [ ] Tests pass after implementation (Green)
- [ ] Code is refactored (Blue)
- [ ] No test duplicates production logic
- [ ] Tests are readable and maintainable
- [ ] Edge cases are covered
- [ ] Security tests pass
- [ ] Performance is acceptable
- [ ] Code coverage > 80%

---

## Summary

**TDD is not optional** in this project. Every feature must follow:

1. **Red:** Write failing test
2. **Green:** Make it pass
3. **Blue:** Refactor
4. **Commit:** With clear message

**Key Metrics:**
- Test coverage: > 80%
- Unit test execution: < 5 seconds
- CI build: < 10 minutes
- No commits without tests

---

**Document Version:** 1.0  
**Last Updated:** 2026-02-27  
**Status:** Active Development Process
