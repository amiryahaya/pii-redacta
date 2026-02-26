# TDD Quick Reference Card

## Daily TDD Workflow

```
┌─────────────────────────────────────────────────────────────┐
│                    DAILY TDD CYCLE                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  1. PICK TASK                                               │
│     └─ From current sprint backlog                          │
│                                                             │
│  2. WRITE TEST (Red)                                        │
│     └─ cargo test test_name                                 │
│     └─ Confirm it FAILS                                     │
│                                                             │
│  3. WRITE CODE (Green)                                      │
│     └─ Minimal code to pass                                 │
│     └─ cargo test test_name                                 │
│     └─ Confirm it PASSES                                    │
│                                                             │
│  4. REFACTOR (Blue)                                         │
│     └─ Improve code quality                                 │
│     └─ cargo test                                           │
│     └─ All tests still PASS                                 │
│                                                             │
│  5. COMMIT                                                  │
│     └─ git add .                                            │
│     └─ git commit -m "type(scope): description"             │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Sprint End Workflow

### Week 2: Code Review

```bash
# 1. Self-review
cargo test --all-features
cargo clippy -- -D warnings
cargo fmt --check
cargo tarpaulin --out Html

# 2. Create PR (if using PR workflow)
git checkout -b sprint/N-name
git push origin sprint/N-name

# 3. Peer review checklist
# [ ] Tests comprehensive
# [ ] Code readable
# [ ] No duplication
# [ ] Error handling complete
# [ ] Security considered
```

### Week 2.5: QA Phase

```bash
# 1. Integration tests
cargo test --test '*'

# 2. Performance tests
cargo bench

# 3. Security audit
cargo audit
cargo deny check

# 4. Documentation
cargo doc --no-deps
```

### Sprint End: Commit & Push

```bash
# 1. Squash commits (if needed)
git rebase -i HEAD~N

# 2. Final commit
git commit -m "feat(scope): sprint N - description

- Change 1
- Change 2

Tests: X tests passing
Coverage: X%"

# 3. Tag
git tag -a sprint/N -m "Sprint N complete"

# 4. Push
git push origin main
git push origin sprint/N
```

---

## Test Commands

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name
cargo test module_name::
cargo test -- --nocapture

# Run ignored tests
cargo test -- --ignored

# Run benchmarks
cargo bench

# Check coverage
cargo tarpaulin --out Html
cargo tarpaulin --out Xml

# Linting
cargo clippy
cargo clippy -- -D warnings

# Format
cargo fmt
cargo fmt -- --check

# Documentation
cargo doc
cargo doc --open
```

---

## Commit Message Template

```
<type>(<scope>): sprint <N> - <short description>

[optional longer description]

- Bullet point of changes
- Another change

Tests: <count> tests passing
Coverage: <percentage>%
```

### Types
| Type | Use For |
|------|---------|
| `feat` | New feature |
| `fix` | Bug fix |
| `refactor` | Code restructuring |
| `test` | Adding tests |
| `docs` | Documentation |
| `perf` | Performance |
| `chore` | Maintenance |

### Scopes
| Scope | Description |
|-------|-------------|
| `core` | Core library |
| `api` | REST API |
| `detection` | Detection engine |
| `tokenization` | Tokenization |
| `extraction` | File extraction |
| `security` | Security features |
| `observability` | Logs, metrics |

---

## Test Structure Template

```rust
// File: src/module/feature.rs

// Implementation here

#[cfg(test)]
mod tests {
    use super::*;
    
    // ============== HAPPY PATH ==============
    
    #[test]
    fn test_feature_success() {
        // Arrange
        let input = "valid input";
        
        // Act
        let result = feature(input);
        
        // Assert
        assert!(result.is_ok());
    }
    
    // ============== EDGE CASES ==============
    
    #[test]
    fn test_feature_empty_input() {
        let result = feature("");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_feature_very_long_input() {
        let input = "x".repeat(1_000_000);
        let result = feature(&input);
        assert!(result.is_ok());
    }
    
    // ============== SECURITY ==============
    
    #[test]
    fn test_no_sensitive_data_in_error() {
        let result = feature("secret");
        let err = result.unwrap_err();
        assert!(!err.to_string().contains("secret"));
    }
}
```

---

## Sprint Checklist

### Daily
- [ ] Tests written before code
- [ ] All tests pass
- [ ] Code committed

### Weekly
- [ ] Coverage checked (>80%)
- [ ] Clippy warnings resolved
- [ ] Documentation updated

### Sprint End
- [ ] Code review complete
- [ ] QA checklist done
- [ ] Commits squashed
- [ ] Tag created
- [ ] Pushed to origin

---

## Emergency Procedures

### Test is Flaky
```bash
# Run multiple times to confirm
cargo test test_name -- --test-threads=1
cargo test test_name --release
```

### Coverage Dropped
```bash
# Check what changed
cargo tarpaulin --out Html --skip-clean
open tarpaulin-report.html
```

### Build Broken
```bash
# Clean and rebuild
cargo clean
cargo build
cargo test
```

---

**Keep this card handy during development!**
