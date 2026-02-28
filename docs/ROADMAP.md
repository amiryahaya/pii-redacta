# PII Redacta Development Roadmap

**Version:** 4.0 (Sprint-Based TDD)
**Last Updated:** 2026-02-28
**Status:** Sprint 10 Complete — Sprint 11 Ready

---

## Sprint Methodology

Each sprint follows: **TDD → Code Review → QA → Commit & Push**

```
Week 1-2: Development (TDD Cycle Daily)
Week 2.5: Code Review + QA
Sprint End: Commit, Tag, Push
```

---

## Phase 1: MVP - Standard Mode (Sprints 1-8) ✅

### Sprint 1: Project Foundation & Core Types ✅
**Sprint Commit:** `feat(core): sprint 1 - project foundation and core types`

### Sprint 2: Pattern-Based Detection Engine ✅
**Sprint Commit:** `feat(detection): sprint 2 - pattern-based detection engine`

### Sprint 3: Tokenization Engine ✅
**Sprint Commit:** `feat(tokenization): sprint 3 - tokenization engine`

### Sprint 4: REST API Foundation ✅
**Sprint Commit:** `feat(api): sprint 4 - REST API foundation`

### Sprint 5: File Processing ✅
**Sprint Commit:** `feat(extraction): sprint 5 - file processing`

### Sprint 6: File Upload API & Integration ✅
**Sprint Commit:** `feat(api): sprint 6 - file upload and async processing`

### Sprint 7: Observability & Documentation ✅
**Sprint Commit:** `feat(observability): sprint 7 - logging, metrics, docs`

### Sprint 8: Security Hardening & MVP Release ✅
**Sprint Commit:** `feat(release): sprint 8 - MVP security hardening and release`

**MVP Tag:** `v0.1.0-mvp`

---

## Phase 2: Portal & User Management (Sprints 9-10) ✅

### Sprint 9: Database, Auth & Configurable Tier System ✅
- PostgreSQL with SQLx migrations (users, tiers, subscriptions, api_keys, usage_logs)
- JWT authentication (register, login, logout, me, profile, preferences)
- Configurable tier system with TierManager + Redis caching
- API key management (HMAC-SHA256, generate, revoke, list)
- Usage & dashboard handlers
- Subscription endpoints
- Portal UI (React 18, Vite, Tailwind, Zustand, TanStack Query)
- Security hardening (4 code review rounds)

**Tests:** 264 passing

### Sprint 10: Portal/API Integration Fixes ✅
- Fixed serde camelCase mismatches (RegisterRequest, ChangePasswordRequest, all response structs)
- Fixed preferences URL path, dashboard field names, usage summary response shape
- Added authenticated `/api/v1/detect` endpoint with fire-and-forget usage logging
- Added in-memory rate limiting (10 req/min) on login/register with eviction
- Wired `usage_logs` table via `record_usage()` in core DB layer
- Fixed portal: logout calls backend, trend logic, email guard, redirect
- Updated `.env.example` to `PII_REDACTA_` prefix

**Tests:** 270 passing

---

## Phase 3: Infrastructure & Security (Sprints 11-12)

### Sprint 11: File Processing Pipeline & Redis Integration ⏳
**Duration:** 2 weeks
**Status:** Ready to start

**Deliverables:**
- [ ] Background job worker (processes uploads: Pending → Processing → Completed/Failed)
- [ ] File extraction integration (PDF, DOCX, XLSX, CSV → detect → results)
- [ ] Redis client integration (replace in-memory rate limiter with distributed)
- [ ] Real Prometheus metrics (replace hardcoded zeros)
- [ ] Redis health check in `health_deep()`
- [ ] API key list pagination (limit/offset)

**Tests Target:** 30 new tests
**Sprint Commit:** `feat(api): sprint 11 - file processing pipeline, Redis integration, metrics`

---

### Sprint 12: Security Hardening v2 🔒
**Duration:** 2 weeks
**Status:** Planned

**Deliverables:**
- [ ] JWT token blacklist on password change (Redis-backed)
- [ ] Admin flag server-side re-verification (query DB on privileged ops)
- [ ] X-Forwarded-For trust chain validation (configurable trusted proxies)
- [ ] Handler unit test coverage (usage, subscription, auth, api_keys)
- [ ] Error tracking integration (Sentry) in portal ErrorBoundary

**Tests Target:** 25 new tests
**Sprint Commit:** `fix(security): sprint 12 - token blacklist, admin verification, proxy trust`

---

## Phase 4: Features & Enterprise (Sprints 13-16)

### Sprint 13: Playground Feature 📋
**Duration:** 2 weeks
**Status:** Planned

**Deliverables:**
- [ ] Playground backend handlers (limits, submit, history)
- [ ] Playground portal page (file upload, results display)
- [ ] Daily quota enforcement from tier limits
- [ ] Playground submission tracking in `usage_logs`

**Tests Target:** 25 new tests
**Sprint Commit:** `feat(api): sprint 13 - authenticated playground`

---

### Sprint 14: Batch Processing & Webhooks 📋
**Duration:** 2 weeks
**Status:** Planned

**Deliverables:**
- [ ] Batch detection endpoint (multi-text, multi-file)
- [ ] Webhook delivery system (configurable per-user)
- [ ] Custom detection rules engine (user-defined regex patterns)

**Tests Target:** 25 new tests
**Sprint Commit:** `feat(api): sprint 14 - batch processing, webhooks, custom rules`

---

### Sprint 15: GLiNER Integration 📋
**Duration:** 2 weeks
**Status:** Planned

**Deliverables:**
- [ ] GLiNER Python microservice integration
- [ ] Malaysian PII entity detection (address, postcode, organization)
- [ ] Hybrid detection pipeline (regex → GLiNER → confidence merge)
- [ ] Fallback to pattern-only when GLiNER unavailable

**Sprint Commit:** `feat(detection): sprint 15 - GLiNER integration`

---

### Sprint 16: Enterprise & Release 📋
**Duration:** 2 weeks
**Status:** Planned

**Deliverables:**
- [ ] Admin dashboard (tier management UI)
- [ ] Billing foundation (Stripe integration)
- [ ] Team/organization accounts
- [ ] Performance optimization & caching
- [ ] Comprehensive documentation

**Release Tag:** `v1.0.0`

---

## Sprint Tracking Board

```
Legend:
✅ Complete
⏳ Ready to start
📋 Planned
```

### Completed
| Sprint | Name | Tests | Key Metric |
|--------|------|-------|------------|
| 1 | Foundation | ✅ | Core types |
| 2 | Detection Engine | ✅ | < 1ms detection |
| 3 | Tokenization | ✅ | Deterministic HMAC |
| 4 | REST API | ✅ | Axum endpoints |
| 5 | File Processing | ✅ | PDF/DOCX/CSV |
| 6 | Upload API | ✅ | Multipart + jobs |
| 7 | Observability | ✅ | Metrics + logging |
| 8 | Security Hardening | ✅ | Headers + limits |
| 9 | Database & Auth | ✅ | 264 tests |
| 10 | Integration Fixes | ✅ | 270 tests |

### Current & Upcoming
| Sprint | Name | Status | Target |
|--------|------|--------|--------|
| 11 | File Pipeline & Redis | ⏳ | Next |
| 12 | Security Hardening v2 | 📋 | After 11 |
| 13 | Playground | 📋 | After 12 |
| 14 | Batch & Webhooks | 📋 | After 13 |
| 15 | GLiNER Integration | 📋 | After 14 |
| 16 | Enterprise & Release | 📋 | Final |

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

**Document Version:** 4.0
**Last Updated:** 2026-02-28
**Next Review:** After Sprint 11
