# PII Redacta - Project Roadmap

## Overview

This document outlines the development roadmap for PII Redacta, from initial setup to production deployment.

**Project Duration:** 16 weeks  
**Team Size:** 3-4 engineers (Backend, Rust, DevOps, QA)  
**MVP Target:** Week 8  
**Production Release:** Week 14

---

## Phase 1: Foundation (Weeks 1-3)

### Week 1: Project Setup & Core Infrastructure

**Goals:**
- Initialize Rust project structure
- Set up CI/CD pipeline
- Establish development environment

**Tasks:**
```markdown
- [ ] Create workspace structure (Cargo workspace with crates)
  - pii_redacta_core (detection + tokenization)
  - pii_redacta_api (gRPC + REST)
  - pii_redacta_db (PostgreSQL layer)
  - pii_redacta_security (crypto + secure memory)
  
- [ ] Set up CI/CD (GitHub Actions)
  - Rustfmt, clippy checks
  - Test execution
  - Security audit (cargo-audit)
  - Docker build
  
- [ ] Configure development environment
  - Docker Compose with PostgreSQL 18
  - Hot reload for development
  - Pre-commit hooks
  
- [ ] Documentation skeleton
  - API documentation (OpenAPI)
  - Architecture Decision Records (ADRs)
```

**Deliverables:**
- Working development environment
- CI/CD pipeline passing
- Project structure committed

---

### Week 2: Security Layer Implementation

**Goals:**
- Implement core cryptographic primitives
- Build secure memory arena
- ML-KEM-768 integration

**Tasks:**
```markdown
- [ ] Cryptographic primitives
  - AES-256-GCM encryption/decryption
  - HKDF-SHA384 key derivation
  - Secure random generation (ChaCha20Rng)
  
- [ ] Post-quantum cryptography
  - ML-KEM-768 key generation
  - Encapsulation/Decapsulation
  - Hybrid KEM (optional KAZ-KEM support)
  
- [ ] Secure memory management
  - mlock/munlock implementation
  - Automatic zeroization on Drop
  - Arena with TTL support
  - Memory pool for frequent allocations
  
- [ ] Constant-time operations
  - Secure comparison (subtle crate)
  - Side-channel resistant code review
```

**Deliverables:**
- All crypto tests passing
- Security audit of crypto code
- Benchmarks for KEM operations

---

### Week 3: Database Layer & Migrations

**Goals:**
- PostgreSQL 18 schema implementation
- SQLx integration with compile-time checks
- Migration system

**Tasks:**
```markdown
- [ ] Database schema implementation
  - Create all tables (tenants, conversations, redacted_files, audit_logs)
  - Indexes and constraints
  - Partitioning for audit_logs
  
- [ ] SQLx integration
  - Query macros with compile-time validation
  - Connection pooling (deadpool)
  - Transaction management
  
- [ ] Migration system
  - sqlx-cli setup
  - Initial migration scripts
  - Rollback procedures
  
- [ ] Repository pattern
  - TenantRepository
  - ConversationRepository
  - RedactedFileRepository
  - AuditLogRepository
```

**Deliverables:**
- Database schema deployed
- All repository tests passing
- Migration documentation

---

## Phase 2: Core Engine (Weeks 4-6)

### Week 4: Pattern Detection Engine

**Goals:**
- Aho-Corasick multi-pattern matcher
- PII pattern definitions
- Detection pipeline foundation

**Tasks:**
```markdown
- [ ] Pattern matching engine
  - Aho-Corasick implementation
  - Regex fallback for complex patterns
  - Pattern priority/conflict resolution
  
- [ ] PII pattern library
  - EMAIL (RFC 5322 compliant)
  - PHONE (Malaysian format + international)
  - NRIC (Malaysian IC)
  - PASSPORT (multiple countries)
  - CREDIT_CARD (Luhn validation)
  - IP_ADDRESS (v4/v6)
  - DATE_OF_BIRTH (multiple formats)
  
- [ ] Confidence scoring
  - Pattern-specific confidence
  - Format validation boosting
  - Context analysis preparation
```

**Deliverables:**
- Pattern detection <1ms P99
- 95%+ accuracy on test dataset
- Comprehensive pattern tests

---

### Week 5: Tokenization Engine

**Goals:**
- Deterministic token generation
- Token map management
- Zero-knowledge token storage

**Tasks:**
```markdown
- [ ] Token generation
  - Deterministic hash: SHA3-256(conversation_id + value)
  - Token format: <<PII_TYPE_HASH>>
  - Collision detection and handling
  
- [ ] Token map management
  - In-memory token map structure
  - Efficient lookups
  - Token map encryption with DEK
  
- [ ] Zero-knowledge storage
  - Client-provided DEK handling
  - ML-KEM key wrapping
  - Encrypted token map serialization
  
- [ ] Token restoration
  - Reverse lookup from token map
  - Batch restoration
  - Streaming restoration for large docs
```

**Deliverables:**
- Tokenization round-trip tests
- Token map encryption verified
- Performance benchmarks

---

### Week 6: Detection Pipeline Integration

**Goals:**
- 4-tier pipeline implementation
- Parallel detection
- Result merging

**Tasks:**
```markdown
- [ ] Pipeline architecture
  - Tier 1: Pattern detection (sync)
  - Tier 2: Presidio integration (async)
  - Tier 3: Document classification (async, optional)
  - Tier 4: Context validation (async, optional)
  
- [ ] Parallel execution
  - tokio::join! for concurrent tiers
  - Cancellation support
  - Timeout handling per tier
  
- [ ] Result merging
  - Overlap detection
  - Confidence-based selection
  - Source prioritization
  
- [ ] Domain-specific rules
  - Rules engine structure
  - Medical domain rules
  - Financial domain rules
  - Legal domain rules
```

**Deliverables:**
- Full pipeline working end-to-end
- Parallel detection benchmarks
- Domain rule tests

---

## Phase 3: API Layer (Weeks 7-8)

### Week 7: gRPC API Implementation

**Goals:**
- Protocol buffer definitions
- Tonic service implementation
- Authentication middleware

**Tasks:**
```markdown
- [ ] Protocol buffers
  - Define all services and messages
  - Versioning strategy (v1, v2 paths)
  - Generate Rust code
  
- [ ] Service implementation
  - DetectionService
  - ConversationService
  - FileService
  - Health service
  
- [ ] Middleware
  - JWT validation (jsonwebtoken)
  - Rate limiting (governor)
  - Request ID propagation
  - Authentication context
  
- [ ] Error handling
  - gRPC status codes
  - Detailed error messages
  - Error logging
```

**Deliverables:**
- gRPC server running
- All service methods implemented
- Integration tests passing

---

### Week 8: REST API & MVP Completion

**Goals:**
- Axum REST API
- OpenAPI documentation
- MVP feature complete

**Tasks:**
```markdown
- [ ] REST endpoints
  - /health, /health/ready
  - /api/v1/detect
  - /api/v1/tokenize
  - /api/v1/conversations
  - /api/v1/files
  
- [ ] Content negotiation
  - JSON request/response
  - Streaming for large files
  - Compression (gzip, brotli)
  
- [ ] Documentation
  - OpenAPI 3.1 spec
  - API examples
  - Error code reference
  
- [ ] MVP validation
  - Feature parity with existing service
  - Performance benchmarks vs existing
  - Security review
```

**Deliverables:**
- MVP complete and deployed to staging
- API documentation published
- Performance report

**Milestone: MVP Release**

---

## Phase 4: Advanced Features (Weeks 9-11)

### Week 9: Presidio & ML Integration

**Goals:**
- Presidio analyzer integration
- ONNX runtime for local NER
- SLM validation (Ollama)

**Tasks:**
```markdown
- [ ] Presidio integration
  - HTTP client for Presidio analyzer
  - Circuit breaker pattern
  - Result transformation
  
- [ ] ONNX Runtime
  - Model loading and inference
  - spaCy model export to ONNX
  - Batch inference optimization
  
- [ ] Ollama integration (optional)
  - Phi-3 for document classification
  - Mistral 7B for context validation
  - Caching layer for SLM results
  - Circuit breaker for Ollama
```

**Deliverables:**
- ML-based detection working
- SLM integration (if enabled)
- Circuit breaker tests

---

### Week 10: File Processing

**Goals:**
- Large file support
- Streaming processing
- Multiple format support

**Tasks:**
```markdown
- [ ] File upload handling
  - Multipart form support
  - Streaming to S3/object storage
  - Progress tracking
  
- [ ] Document parsers
  - Plain text
  - PDF (text extraction)
  - Word documents
  - Excel/CSV
  
- [ ] Background processing
  - Oban job queue integration
  - Progress reporting
  - Error handling and retry
  
- [ ] Redacted file generation
  - Replace PII in original format
  - Preserve formatting
  - Encrypted storage
```

**Deliverables:**
- File upload and processing
- Background job system
- Format support tests

---

### Week 11: Monitoring & Observability

**Goals:**
- Metrics collection
- Distributed tracing
- Alerting setup

**Tasks:**
```markdown
- [ ] Prometheus metrics
  - Detection latency histograms
  - Entity counters
  - Token map size gauges
  - Database connection metrics
  
- [ ] Distributed tracing
  - OpenTelemetry integration
  - Trace correlation
  - Span annotations
  
- [ ] Structured logging
  - JSON format
  - Context propagation
  - Sensitive data scrubbing
  
- [ ] Health checks
  - Deep health (database connectivity)
  - Dependency health
  - Custom health indicators
```

**Deliverables:**
- Grafana dashboards
- Alerting rules
- Runbooks for common issues

---

## Phase 5: Production Readiness (Weeks 12-14)

### Week 12: Security Hardening

**Goals:**
- Security audit
- Penetration testing
- Compliance documentation

**Tasks:**
```markdown
- [ ] Security testing
  - Fuzzing detection inputs
  - SQL injection testing
  - Authentication bypass attempts
  
- [ ] Penetration testing
  - Third-party security assessment
  - Vulnerability remediation
  
- [ ] Compliance
  - PDPA compliance documentation
  - GDPR compliance checklist
  - Data handling procedures
  
- [ ] Secret management
  - Vault integration
  - Key rotation procedures
  - Certificate management
```

**Deliverables:**
- Security audit report
- Pen test remediation
- Compliance documentation

---

### Week 13: Performance Optimization

**Goals:**
- Performance tuning
- Load testing
- Capacity planning

**Tasks:**
```markdown
- [ ] Optimization
  - Profile and optimize hot paths
  - Database query optimization
  - Memory allocation reduction
  
- [ ] Load testing
  - k6 or Locust scripts
  - 10,000 req/sec target
  - Identify bottlenecks
  
- [ ] Capacity planning
  - Resource usage modeling
  - Scaling guidelines
  - Cost estimation
  
- [ ] Caching layer
  - Redis for token maps (optional)
  - Detection result caching
  - Cache invalidation strategy
```

**Deliverables:**
- Performance benchmark report
- Load test results
- Scaling documentation

---

### Week 14: Production Deployment

**Goals:**
- Production deployment
- Migration from existing service
- Go-live support

**Tasks:**
```markdown
- [ ] Deployment
  - Kubernetes manifests
  - Helm charts
  - Terraform infrastructure
  
- [ ] Migration
  - Database migration scripts
  - Compatibility layer testing
  - Phased rollout plan
  
- [ ] Cutover
  - Feature flags for gradual migration
  - Rollback procedures
  - 24/7 support rotation
  
- [ ] Post-launch
  - Monitoring dashboards
  - On-call runbooks
  - Incident response plan
```

**Deliverables:**
- Production service live
- Migration complete
- Support documentation

**Milestone: Production Release**

---

## Phase 6: Post-Launch (Weeks 15-16)

### Week 15: Stabilization

**Goals:**
- Bug fixes
- Performance monitoring
- User feedback integration

**Tasks:**
```markdown
- [ ] Monitoring
  - Daily performance reviews
  - Error rate tracking
  - Capacity monitoring
  
- [ ] Bug fixes
  - P0/P1 issues resolution
  - Edge case handling
  - Stability improvements
  
- [ ] Documentation
  - User guides
  - Troubleshooting guides
  - API client examples
```

---

### Week 16: Feature Enhancements

**Goals:**
- New PII types
- Additional ML models
- Performance improvements

**Tasks:**
```markdown
- [ ] Additional PII types
  - International formats
  - Industry-specific PII
  - Custom pattern support
  
- [ ] Enhancements
  - Additional SLM models
  - Improved confidence scoring
  - Batch processing API
  
- [ ] Future planning
  - v2 API design
  - Feature roadmap
  - Technical debt review
```

---

## Risk Management

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| ML-KEM library issues | Medium | High | Have BoringSSL fallback |
| Presidio integration complexity | Medium | Medium | Early prototype in Week 4 |
| Performance not meeting targets | Low | High | Benchmark early, optimize continuously |
| Security vulnerabilities | Low | Critical | Security audit in Week 12 |
| Team member availability | Medium | Medium | Cross-training, documentation |

---

## Success Criteria

### Technical Metrics

| Metric | Target |
|--------|--------|
| Detection latency P99 | <1ms (patterns), <50ms (full) |
| Memory usage | <512MB peak |
| Throughput | 10,000 req/sec per core |
| Uptime | 99.99% |
| Tokenization accuracy | 99.9% |

### Business Metrics

| Metric | Target |
|--------|--------|
| Zero PII leaks | 100% compliance |
| Migration downtime | <5 minutes |
| Cost reduction vs Elixir | 30% lower resource usage |
| Developer satisfaction | >4/5 rating |

---

## Appendix

### A. Development Team Structure

| Role | Responsibility |
|------|----------------|
| Tech Lead | Architecture, code review, technical decisions |
| Rust Engineer | Core engine, security layer |
| Backend Engineer | API, database, integrations |
| DevOps Engineer | Infrastructure, CI/CD, monitoring |
| QA Engineer | Testing strategy, automation |

### B. Technology Stack Summary

| Layer | Technology |
|-------|------------|
| Language | Rust 1.75+ |
| gRPC | Tonic |
| REST | Axum |
| Database | PostgreSQL 18 + sqlx |
| Crypto | aes-gcm, ml-kem, hkdf |
| Async Runtime | Tokio |
| Testing | cargo test, insta |
| Metrics | prometheus, opentelemetry |

### C. Key Dependencies

```toml
[dependencies]
# Core
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }

# API
tonic = "0.11"
axum = "0.7"
tower = "0.4"

# Database
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio"] }
deadpool = "0.10"

# Crypto
aes-gcm = "0.10"
ml-kem = "0.1"
hkdf = "0.12"
sha2 = "0.10"
zeroize = "1.7"

# Detection
aho-corasick = "1.1"
regex = "1.10"
reqwest = { version = "0.11", features = ["json"] }

# Auth
jsonwebtoken = "9"
rustls = "0.22"

# Observability
tracing = "0.1"
opentelemetry = "0.21"
prometheus = "0.13"
```

---

**Document Version:** 1.0  
**Last Updated:** 2026-02-27  
**Status:** Draft
