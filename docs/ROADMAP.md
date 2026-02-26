# PII Redacta Development Roadmap

**Version:** 3.0 (Sprint-Based TDD)  
**Last Updated:** 2026-02-27  
**Status:** Ready for Sprint 1

---

## Sprint Methodology

Each sprint follows: **TDD → Code Review → QA → Commit & Push**

```
Week 1-2: Development (TDD Cycle Daily)
Week 2.5: Code Review + QA
Sprint End: Commit, Tag, Push
```

---

## Phase 1: MVP - Standard Mode (Sprints 1-8)

### Sprint 1: Project Foundation & Core Types ⏳
**Duration:** 2 weeks  
**Dates:** Week 1-2  
**Status:** Ready to start

**TDD Deliverables:**
- [ ] Workspace structure tests
- [ ] Domain type tests (Entity, EntityType, DetectionResult)
- [ ] Error type tests
- [ ] CI/CD workflow tests

**Code Review Checklist:**
- [ ] All tests pass
- [ ] Coverage > 80%
- [ ] Types properly serializable
- [ ] CI configured

**QA Checklist:**
- [ ] Clean build in fresh environment
- [ ] No clippy warnings
- [ ] Documentation builds

**Sprint Commit:** `feat(core): sprint 1 - project foundation and core types`

---

### Sprint 2: Pattern-Based Detection Engine 📋
**Duration:** 2 weeks  
**Dates:** Week 3-4  
**Status:** Planned

**TDD Deliverables:**
- [ ] Email pattern detection tests
- [ ] Malaysian NRIC pattern tests
- [ ] Phone number pattern tests
- [ ] Credit card pattern tests
- [ ] Performance tests (< 1ms)
- [ ] Benchmark tests

**Code Review Checklist:**
- [ ] All patterns tested
- [ ] Performance targets met
- [ ] Benchmarks exist
- [ ] No regex DoS vulnerabilities

**QA Checklist:**
- [ ] Benchmarks pass
- [ ] Memory stable under load
- [ ] Unicode handling correct

**Sprint Commit:** `feat(detection): sprint 2 - pattern-based detection engine`

---

### Sprint 3: Tokenization Engine 📋
**Duration:** 2 weeks  
**Dates:** Week 5-6  
**Status:** Planned

**TDD Deliverables:**
- [ ] Token generator determinism tests
- [ ] Tenant isolation tests
- [ ] Tokenizer tests
- [ ] Overlapping entity tests
- [ ] Token store tests (with TTL)
- [ ] Detokenization tests

**Code Review Checklist:**
- [ ] Tokens are deterministic
- [ ] Tenant isolation works
- [ ] No PII in tokens
- [ ] Overlaps handled

**QA Checklist:**
- [ ] Detokenize exact restore
- [ ] TTL expiration works
- [ ] Memory bounded

**Sprint Commit:** `feat(tokenization): sprint 3 - tokenization engine`

---

### Sprint 4: REST API Foundation 📋
**Duration:** 2 weeks  
**Dates:** Week 7-8  
**Status:** Planned

**TDD Deliverables:**
- [ ] Health endpoint tests
- [ ] Detection endpoint tests
- [ ] Error handling tests
- [ ] Request validation tests
- [ ] Content-type tests

**Code Review Checklist:**
- [ ] All endpoints tested
- [ ] Errors comprehensive
- [ ] Status codes correct
- [ ] API types validated

**QA Checklist:**
- [ ] Response time < 50ms
- [ ] Concurrent requests work
- [ ] JSON correct

**Sprint Commit:** `feat(api): sprint 4 - REST API foundation`

---

### Sprint 5: File Processing 📋
**Duration:** 2 weeks  
**Dates:** Week 9-10  
**Status:** Planned

**TDD Deliverables:**
- [ ] Text extraction tests
- [ ] PDF extraction tests
- [ ] DOCX extraction tests
- [ ] MIME type detection tests
- [ ] Unsupported type tests

**Code Review Checklist:**
- [ ] All formats tested
- [ ] Error handling robust
- [ ] Binary safety verified

**QA Checklist:**
- [ ] PDF extraction accurate
- [ ] DOCX extraction preserves text
- [ ] Large files handled

**Sprint Commit:** `feat(extraction): sprint 5 - file processing`

---

### Sprint 6: File Upload API & Integration 📋
**Duration:** 2 weeks  
**Dates:** Week 11-12  
**Status:** Planned

**TDD Deliverables:**
- [ ] Multipart upload tests
- [ ] Job queue tests
- [ ] Job status tests
- [ ] File size limit tests
- [ ] Async processing tests

**Code Review Checklist:**
- [ ] Upload secure
- [ ] Queue reliable
- [ ] Status tracking works

**QA Checklist:**
- [ ] Large uploads work
- [ ] Queue handles load
- [ ] Jobs complete

**Sprint Commit:** `feat(api): sprint 6 - file upload and async processing`

---

### Sprint 7: Observability & Documentation 📋
**Duration:** 2 weeks  
**Dates:** Week 13-14  
**Status:** Planned

**TDD Deliverables:**
- [ ] PII-safe logging tests
- [ ] Metrics collection tests
- [ ] OpenAPI spec tests
- [ ] Health metrics tests

**Code Review Checklist:**
- [ ] No PII in logs
- [ ] Metrics accurate
- [ ] OpenAPI valid

**QA Checklist:**
- [ ] Logs redact PII
- [ ] Prometheus metrics work
- [ ] OpenAPI generates correctly

**Sprint Commit:** `feat(observability): sprint 7 - logging, metrics, docs`

---

### Sprint 8: Security Hardening & MVP Release 📋
**Duration:** 2 weeks  
**Dates:** Week 15-16  
**Status:** Planned

**TDD Deliverables:**
- [ ] Rate limiting tests
- [ ] Security header tests
- [ ] Input validation tests
- [ ] Docker build tests
- [ ] Security audit tests

**Code Review Checklist:**
- [ ] Rate limits enforced
- [ ] Security headers present
- [ ] Docker image builds
- [ ] No security issues

**QA Checklist:**
- [ ] Rate limiting works
- [ ] Security scan passes
- [ ] Docker runs correctly
- [ ] All tests pass

**Sprint Commit:** `feat(release): sprint 8 - MVP security hardening and release`

**MVP Tag:** `v0.1.0-mvp`

---

## Phase 2: Enhanced Detection (Sprints 9-12)

### Sprint 9: GLiNER Integration 📋
**Duration:** 2 weeks  
**Dates:** Week 17-18  
**Status:** Planned

**TDD Deliverables:**
- [ ] GLiNER binding tests
- [ ] Malaysian PII tests
- [ ] Fallback to patterns tests
- [ ] Performance tests

**Sprint Commit:** `feat(detection): sprint 9 - GLiNER integration`

---

### Sprint 10: Presidio Integration 📋
**Duration:** 2 weeks  
**Dates:** Week 19-20  
**Status:** Planned

**TDD Deliverables:**
- [ ] Presidio binding tests
- [ ] Context analysis tests
- [ ] Score aggregation tests

**Sprint Commit:** `feat(detection): sprint 10 - Presidio integration`

---

### Sprint 11: Detection Pipeline V2 📋
**Duration:** 2 weeks  
**Dates:** Week 21-22  
**Status:** Planned

**TDD Deliverables:**
- [ ] 3-tier pipeline tests
- [ ] Tier selection tests
- [ ] Confidence scoring tests

**Sprint Commit:** `feat(detection): sprint 11 - enhanced detection pipeline`

---

### Sprint 12: Performance Optimization 📋
**Duration:** 2 weeks  
**Dates:** Week 23-24  
**Status:** Planned

**TDD Deliverables:**
- [ ] Benchmark regression tests
- [ ] Memory optimization tests
- [ ] Concurrency tests

**Sprint Commit:** `perf(core): sprint 12 - performance optimization`

---

## Phase 3: Enterprise Features (Sprints 13-16)

### Sprint 13: Zero-Knowledge Architecture 📋
**Duration:** 2 weeks  
**Status:** Planned

**TDD Deliverables:**
- [ ] ML-KEM tests
- [ ] Hybrid encryption tests
- [ ] Secure processing tests

**Sprint Commit:** `feat(security): sprint 13 - zero-knowledge architecture`

---

### Sprint 14: gRPC API 📋
**Duration:** 2 weeks  
**Status:** Planned

**TDD Deliverables:**
- [ ] gRPC service tests
- [ ] Streaming tests
- [ ] Proto validation tests

**Sprint Commit:** `feat(api): sprint 14 - gRPC API`

---

### Sprint 15: PostgreSQL Integration 📋
**Duration:** 2 weeks  **Status:** Planned

**TDD Deliverables:**
- [ ] Migration tests
- [ ] Job storage tests
- [ ] Audit log tests

**Sprint Commit:** `feat(storage): sprint 15 - PostgreSQL integration`

---

### Sprint 16: Monitoring & Enterprise Release 📋
**Duration:** 2 weeks  
**Status:** Planned

**TDD Deliverables:**
- [ ] Grafana dashboard tests
- [ ] Alerting tests
- [ ] Enterprise integration tests

**Sprint Commit:** `feat(release): sprint 16 - enterprise release`

**Release Tag:** `v1.0.0`

---

## Sprint Tracking Board

```
Legend:
⏳ Ready to start
🔄 In progress
✅ Complete
📋 Planned
```

### Current Sprint
| Sprint | Name | Status | Branch | Tests | Coverage |
|--------|------|--------|--------|-------|----------|
| 1 | Foundation | ⏳ | `sprint/1-foundation` | 0/24 | 0% |

### Upcoming Sprints
| Sprint | Name | Status | Target |
|--------|------|--------|--------|
| 2 | Detection Engine | 📋 | Week 3-4 |
| 3 | Tokenization | 📋 | Week 5-6 |
| 4 | REST API | 📋 | Week 7-8 |
| 5 | File Processing | 📋 | Week 9-10 |
| 6 | Upload API | 📋 | Week 11-12 |
| 7 | Observability | 📋 | Week 13-14 |
| 8 | MVP Release | 📋 | Week 15-16 |

---

## Sprint Commit Log

| Sprint | Commit Hash | Message | Tests | Coverage |
|--------|-------------|---------|-------|----------|
| - | - | Initial setup | - | - |

---

## Definition of Done

### Per Sprint
- [ ] All TDD cycles complete (Red → Green → Blue)
- [ ] Unit tests > 80% coverage
- [ ] Integration tests pass
- [ ] Code review approved
- [ ] QA checklist complete
- [ ] Commits squashed and pushed
- [ ] Sprint tagged

### Per Phase
- [ ] All sprints complete
- [ ] Integration tests pass end-to-end
- [ ] Performance benchmarks met
- [ ] Security audit passed
- [ ] Documentation complete
- [ ] Release tagged

---

## Velocity Tracking

| Sprint | Planned | Completed | Velocity |
|--------|---------|-----------|----------|
| 1 | 24 tests | - | - |

---

**Document Version:** 3.0  
**Last Updated:** 2026-02-27  
**Next Review:** After Sprint 1
