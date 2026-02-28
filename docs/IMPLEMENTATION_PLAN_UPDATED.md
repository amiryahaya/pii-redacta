# PII Redacta - Updated Implementation Plan

**Version:** 5.0 (Consolidated)
**Last Updated:** 2026-02-28
**Status:** Sprints 1-10 Complete, Sprint 11 Ready

---

## Executive Summary

### What We've Built (Sprints 1-10) ✅
- Pattern-based PII detection (Email, NRIC, Phone, Credit Card) — < 1ms
- Deterministic tokenization with HMAC-SHA256, tenant-isolated
- REST API with Axum (health, detect, upload, jobs, metrics)
- File extraction (TXT, PDF, DOCX, XLSX, CSV)
- PostgreSQL database (users, tiers, subscriptions, api_keys, usage_logs)
- JWT authentication (register, login, logout, profile, preferences)
- Configurable tier system with TierManager + Redis caching
- API key management (HMAC-SHA256 hashed, env-prefixed)
- Authenticated detection endpoint with usage logging
- In-memory rate limiting on auth endpoints
- Portal UI (React 18 + TypeScript, Vite, Tailwind, Zustand, TanStack Query)
- Dashboard, Usage, API Keys, Settings portal pages
- 270 integration + unit tests passing

### What We're Building Next (Sprints 11-16)
- **Sprint 11:** File processing pipeline, Redis integration, real metrics
- **Sprint 12:** Security hardening v2 (token blacklist, admin verification)
- **Sprint 13:** Authenticated playground (backend + portal)
- **Sprint 14:** Batch processing, webhooks, custom rules
- **Sprint 15:** GLiNER integration (ML-based detection)
- **Sprint 16:** Enterprise features & v1.0.0 release

---

## Current State

### Test Summary
| Suite | Count |
|-------|-------|
| Core unit tests | 70 |
| API unit tests | 60 |
| API key integration | 23 |
| Auth API integration | 30 |
| Auth integration | 7 |
| E2E integration | 32 |
| API key manager | 15 |
| Project structure | 6 |
| Tier manager | 20 |
| Doc tests | 2 (ignored) |
| **Total** | **270** |

### Known Gaps (Items 1-14)
| # | Issue | Priority | Sprint |
|---|-------|----------|--------|
| 1 | File upload processing pipeline stubbed (jobs stay Pending) | P0 | 11 |
| 2 | Redis not wired (in-memory rate limiter, no distributed state) | P0 | 11 |
| 3 | Metrics endpoint returns hardcoded zeros | P1 | 11 |
| 4 | JWT not invalidated on password change | P1 | 12 |
| 5 | Admin flag trusted from JWT without re-verification | P1 | 12 |
| 6 | X-Forwarded-For spoofable (no proxy trust chain) | P2 | 12 |
| 7 | Playground feature not implemented (referenced in tier config) | P2 | 13 |
| 8 | Batch processing not implemented | P2 | 14 |
| 9 | Webhooks not implemented | P2 | 14 |
| 10 | Custom detection rules not implemented | P2 | 14 |
| 11 | API key list has no pagination | P3 | 11 |
| 12 | Redis health check returns None | P3 | 11 |
| 13 | Portal ErrorBoundary has no error tracking | P3 | 12 |
| 14 | Handler unit test coverage gaps | P3 | 12 |

---

## Sprint 11: File Processing Pipeline & Redis Integration

### Overview
Make the file upload pipeline functional end-to-end and integrate Redis for distributed state. Replace stub metrics with real Prometheus counters.

### 11.1 Background Job Worker

**Problem:** Upload handler accepts files and creates jobs, but jobs never process — they stay `Pending` forever. There is no background worker.

**Files:**
- `crates/pii_redacta_api/src/handlers/upload.rs` — existing upload handler
- `crates/pii_redacta_api/src/handlers/mod.rs` — `JobQueue` (in-memory `DashMap`)
- `crates/pii_redacta_core/src/extraction/` — existing file extractors

**Implementation:**

1. **Create `JobProcessor` struct** in `crates/pii_redacta_api/src/jobs/processor.rs`
   ```rust
   pub struct JobProcessor {
       queue: Arc<JobQueue>,
       db: Arc<Database>,
   }

   impl JobProcessor {
       /// Spawn background task that polls for pending jobs
       pub fn start(self) -> JoinHandle<()> {
           tokio::spawn(async move {
               loop {
                   self.process_next().await;
                   tokio::time::sleep(Duration::from_millis(500)).await;
               }
           })
       }

       async fn process_next(&self) {
           // Find oldest Pending job, transition to Processing
           // Extract text from file bytes using core extractors
           // Run PatternDetector on extracted text
           // Optionally tokenize if redact=true
           // Store results, transition to Completed
           // Record usage via record_usage()
           // On error: transition to Failed with error message
       }
   }
   ```

2. **Update `Job` struct** — add `result` field for detection output:
   ```rust
   pub struct Job {
       pub id: Uuid,
       pub status: JobStatus, // Pending → Processing → Completed | Failed
       pub file_name: String,
       pub file_data: Vec<u8>,
       pub mime_type: String,
       pub created_at: DateTime<Utc>,
       pub result: Option<JobResult>,
       pub error: Option<String>,
   }

   pub struct JobResult {
       pub entities: Vec<Entity>,
       pub processing_time_ms: f64,
       pub redacted_text: Option<String>,
       pub page_count: Option<i32>,
   }
   ```

3. **Update `get_job_status` handler** — return results when job is Completed

4. **Start processor in `main.rs`** — spawn alongside HTTP server with graceful shutdown

**Tests:**
- `test_job_processes_txt_file` — upload TXT → poll until Completed → verify entities
- `test_job_processes_pdf_file` — upload PDF → verify extraction works
- `test_job_failed_state` — upload corrupted file → verify Failed status + error
- `test_job_result_includes_entities` — verify detection results in response
- `test_concurrent_job_processing` — multiple uploads process correctly

### 11.2 Redis Client Integration

**Problem:** Rate limiter uses in-memory `HashMap` (no horizontal scaling). Config loads `REDIS_URL` but no Redis client exists.

**Files:**
- `crates/pii_redacta_api/src/lib.rs` — `AppState`
- `crates/pii_redacta_api/src/middleware/rate_limit.rs` — current in-memory limiter
- `crates/pii_redacta_core/src/db/mod.rs` — add Redis pool

**Implementation:**

1. **Create Redis connection pool** in `crates/pii_redacta_core/src/db/redis.rs`:
   ```rust
   pub struct RedisPool {
       pool: deadpool_redis::Pool,
   }

   impl RedisPool {
       pub async fn new(url: &str) -> Result<Self, RedisError>;
       pub async fn get(&self) -> Result<Connection, RedisError>;
       pub async fn health_check(&self) -> Result<(), RedisError>;
   }
   ```

2. **Add `RedisPool` to `AppState`** as `Option<Arc<RedisPool>>` (graceful degradation — falls back to in-memory when Redis unavailable)

3. **Update `InMemoryRateLimiter`** to `RateLimiter` with two backends:
   ```rust
   pub enum RateLimiter {
       InMemory(InMemoryRateLimiter),
       Redis(RedisRateLimiter),
   }
   ```
   Redis backend uses `INCR` + `EXPIRE` for atomic sliding window.

4. **Update `health_deep()`** — call `redis_pool.health_check()` and report status

**Tests:**
- `test_redis_rate_limiter_allows_within_limit` (requires Redis)
- `test_redis_rate_limiter_blocks_over_limit` (requires Redis)
- `test_rate_limiter_fallback_to_in_memory` — when Redis unavailable
- `test_health_deep_reports_redis_status`

### 11.3 Real Prometheus Metrics

**Problem:** `/metrics` endpoint returns hardcoded zeros. No actual counters or histograms.

**Files:**
- `crates/pii_redacta_api/src/handlers/metrics.rs` — replace stub
- `crates/pii_redacta_api/src/lib.rs` — add metrics registry to AppState

**Implementation:**

1. **Add `metrics` + `metrics-exporter-prometheus` crates** to Cargo.toml

2. **Define metrics in `AppState`:**
   ```rust
   // Counters
   counter!("pii_detection_requests_total", "type" => "text");
   counter!("pii_detection_requests_total", "type" => "file");
   counter!("pii_detection_entities_total", "entity_type" => "email");

   // Histograms
   histogram!("pii_detection_duration_seconds");
   histogram!("pii_upload_file_size_bytes");

   // Gauges
   gauge!("pii_active_jobs");
   ```

3. **Instrument handlers** — increment counters in `detect()`, `detect_authenticated()`, `upload()`

4. **Replace `get_metrics()` handler** — use `metrics-exporter-prometheus` to render Prometheus text format

**Tests:**
- `test_metrics_incremented_on_detect`
- `test_metrics_histogram_records_duration`
- `test_metrics_endpoint_returns_prometheus_format`

### 11.4 API Key Pagination

**Problem:** `list_api_keys` returns all keys with no pagination.

**Files:**
- `crates/pii_redacta_api/src/handlers/api_keys.rs` — `list_api_keys()`

**Implementation:**

1. Add `PaginationParams` query extractor:
   ```rust
   #[derive(Deserialize)]
   pub struct PaginationParams {
       pub limit: Option<i64>,  // default 20, max 100
       pub offset: Option<i64>, // default 0
   }
   ```

2. Update `list_user_keys()` in core to accept limit/offset
3. Return wrapped response with `{ data: [...], total: N, limit: N, offset: N }`

**Tests:**
- `test_list_api_keys_default_pagination`
- `test_list_api_keys_custom_limit`
- `test_list_api_keys_offset`

### Sprint 11 Summary

| Component | New Tests | Files Modified | Files Created |
|-----------|-----------|---------------|---------------|
| Job Processor | 5 | upload.rs, mod.rs, main.rs | jobs/processor.rs, jobs/mod.rs |
| Redis Integration | 4 | lib.rs, rate_limit.rs, health.rs | db/redis.rs |
| Prometheus Metrics | 3 | metrics.rs, lib.rs, Cargo.toml | — |
| API Key Pagination | 3 | api_keys.rs | — |
| **Total** | **15** | **8** | **3** |

**Target tests:** 285 total (270 + 15)

---

## Sprint 12: Security Hardening v2

### Overview
Close security gaps identified in Sprint 10 code review. Add token blacklist, admin verification, proxy trust, error tracking, and handler test coverage.

### 12.1 JWT Token Blacklist on Password Change

**Problem:** After password change, existing JWT tokens remain valid for up to 24 hours. A compromised token cannot be revoked.

**Files:**
- `crates/pii_redacta_api/src/handlers/auth.rs` — `change_password()`
- `crates/pii_redacta_api/src/jwt.rs` — `validate_token()`
- `crates/pii_redacta_api/src/auth/middleware.rs` — JWT verification middleware

**Implementation:**

1. **Add `password_changed_at` column** to `users` table (new migration):
   ```sql
   ALTER TABLE users ADD COLUMN password_changed_at TIMESTAMPTZ;
   ```

2. **Add `iat` (issued-at) claim** to JWT if not already present

3. **On password change:** update `password_changed_at = NOW()` in DB

4. **In auth middleware:** after JWT validation, check `iat < password_changed_at` → reject with 401

5. **Redis-backed blacklist (optional enhancement):**
   - On password change: `SET blacklist:{user_id} {timestamp} EX 86400`
   - In middleware: `GET blacklist:{user_id}` → if exists and `iat < value` → reject
   - Faster than DB query per request

**Tests:**
- `test_old_token_rejected_after_password_change`
- `test_new_token_works_after_password_change`
- `test_password_changed_at_updated`

### 12.2 Admin Flag Server-Side Verification

**Problem:** `is_admin` is trusted directly from JWT claims without DB verification. Forged JWT (if secret leaked) grants admin access.

**Files:**
- `crates/pii_redacta_api/src/auth/middleware.rs`
- `crates/pii_redacta_api/src/extractors.rs` — `AuthUser`

**Implementation:**

1. **For privileged operations only** (not every request — would be too expensive):
   - Create `AdminUser` extractor that re-verifies `is_admin` from DB
   - Use it on admin-only routes (tier management, user management)

2. **`AuthUser` remains JWT-only** for normal authenticated routes (performance)

3. **Cache admin status in Redis** with short TTL (60s) to avoid per-request DB queries

**Tests:**
- `test_admin_extractor_verifies_db`
- `test_non_admin_rejected_from_admin_route`
- `test_admin_status_cached`

### 12.3 X-Forwarded-For Trust Chain

**Problem:** Rate limiter trusts `X-Forwarded-For` header from any client, allowing rate limit bypass.

**Files:**
- `crates/pii_redacta_api/src/lib.rs` — `login_rate_limit_middleware()`
- `crates/pii_redacta_api/src/config/mod.rs` — add `trusted_proxies` config

**Implementation:**

1. **Add `PII_REDACTA_TRUSTED_PROXIES` env var** — comma-separated CIDR ranges (e.g. `10.0.0.0/8,172.16.0.0/12`)
2. **In rate limit middleware:**
   - If `ConnectInfo` IP is in trusted proxies → use rightmost untrusted IP from `X-Forwarded-For`
   - If `ConnectInfo` IP is NOT in trusted proxies → use `ConnectInfo` IP directly (ignore header)
   - If no `ConnectInfo` available → skip rate limiting (existing behavior)
3. **Default:** empty trusted proxies = always use `ConnectInfo` IP (safe default)

**Tests:**
- `test_trusted_proxy_uses_forwarded_for`
- `test_untrusted_client_ignores_forwarded_for`
- `test_empty_trusted_proxies_uses_connect_info`

### 12.4 Handler Unit Tests

**Problem:** Usage, subscription, auth handlers have integration tests but no handler-level unit tests for business logic.

**Files:** Test files alongside each handler

**Tests to add:**
- `usage.rs` — test range parsing ("7d"/"30d"/"90d"), change % calculation, quota computation
- `subscription.rs` — test response serialization, status filtering
- `auth.rs` — test email normalization edge cases, password validation boundaries
- `api_keys.rs` — test name validation, environment parsing, expires_days bounds

**Target:** 12 new unit tests

### 12.5 Portal Error Tracking

**Problem:** `ErrorBoundary.tsx` catches errors but has no reporting.

**Files:**
- `portal/src/components/ErrorBoundary.tsx`
- `portal/package.json` — add `@sentry/react`

**Implementation:**
1. Add Sentry React SDK
2. Initialize in `main.tsx` with `PII_REDACTA_SENTRY_DSN` env var
3. Report caught errors in ErrorBoundary
4. Add breadcrumbs for API calls in Axios interceptor

### Sprint 12 Summary

| Component | New Tests | Files Modified | Files Created |
|-----------|-----------|---------------|---------------|
| Token Blacklist | 3 | auth.rs, jwt.rs, middleware.rs | migration |
| Admin Verification | 3 | middleware.rs, extractors.rs | — |
| Proxy Trust | 3 | lib.rs, config/mod.rs | — |
| Handler Unit Tests | 12 | 4 test files | — |
| Error Tracking | 0 | ErrorBoundary.tsx, main.tsx | — |
| **Total** | **21** | **9+** | **1** |

**Target tests:** 306 total (285 + 21)

---

## Sprint 13: Playground Feature

### Overview
Build an authenticated playground where users can test PII detection on files and text through the portal, with daily quota enforcement from tier limits.

### 13.1 Playground Backend

**Files:**
- `crates/pii_redacta_api/src/handlers/playground.rs` (new)
- `crates/pii_redacta_api/src/lib.rs` — add routes

**Endpoints:**
```
GET  /api/v1/playground/limits    → { dailyRemaining, maxFileSize, maxDaily }
POST /api/v1/playground/submit    → { entities, redactedText, processingTimeMs }
GET  /api/v1/playground/history   → { submissions: [...], total }
```

**Implementation:**

1. **`get_playground_limits()`** — query tier limits for `playground_max_daily` and `playground_max_file_size`, count today's submissions from `usage_logs`

2. **`submit_playground()`:**
   - Accept multipart form data OR JSON text input
   - Check daily quota (`playground_max_daily` from tier)
   - Check file size (`playground_max_file_size` from tier)
   - Extract text (if file), run detection, optionally redact
   - Record to `usage_logs` with `request_type = "playground"`
   - Return results immediately (synchronous, no job queue)

3. **`get_playground_history()`** — query `usage_logs WHERE request_type = 'playground'` with pagination

**Tests:**
- `test_playground_limits_returns_quota`
- `test_playground_submit_text`
- `test_playground_submit_file`
- `test_playground_daily_limit_enforced`
- `test_playground_file_size_limit_enforced`
- `test_playground_history_paginated`
- `test_playground_requires_auth`

### 13.2 Playground Portal Page

**Files:**
- `portal/src/pages/PlaygroundPage.tsx` (new)
- `portal/src/lib/api.ts` — add `playgroundApi`
- `portal/src/types/index.ts` — add types

**UI Components:**
- Daily quota indicator bar
- Text input area (paste or type)
- File drop zone (drag & drop or browse)
- Redact toggle checkbox
- Results panel with tabs: Detected PII | Redacted Text
- History table (recent submissions)

**Route:** `/playground` added to sidebar navigation

### Sprint 13 Summary

| Component | New Tests | Files Modified | Files Created |
|-----------|-----------|---------------|---------------|
| Playground Backend | 7 | lib.rs | handlers/playground.rs |
| Playground Portal | 0 (manual) | api.ts, types/index.ts, Layout.tsx | PlaygroundPage.tsx |
| **Total** | **7** | **3** | **2** |

**Target tests:** 313 total (306 + 7)

---

## Sprint 14: Batch Processing, Webhooks & Custom Rules

### Overview
Implement the three feature flags referenced in tier configuration: batch processing, webhook delivery, and custom detection rules.

### 14.1 Batch Detection Endpoint

**Files:**
- `crates/pii_redacta_api/src/handlers/batch.rs` (new)

**Endpoint:**
```
POST /api/v1/detect/batch → { results: [{ text, entities, processingTimeMs }] }
```

**Implementation:**
- Accept `{ texts: [string], options: { redact, tenantId } }`
- Check tier `batch_processing` feature flag
- Process all texts concurrently with `futures::join_all`
- Record aggregate usage
- Enforce per-request text count limit (from tier)

### 14.2 Webhook System

**Files:**
- `crates/pii_redacta_api/src/handlers/webhooks.rs` (new)
- `crates/pii_redacta_api/src/jobs/webhook_delivery.rs` (new)
- New migration: `webhook_endpoints` table

**Endpoints:**
```
POST   /api/v1/webhooks          → create webhook endpoint
GET    /api/v1/webhooks          → list user's webhooks
DELETE /api/v1/webhooks/{id}     → delete webhook
GET    /api/v1/webhooks/{id}/logs → delivery history
```

**Implementation:**
- Users register webhook URLs with event filters (e.g. `job.completed`, `detection.complete`)
- On relevant events, queue webhook delivery via `tokio::spawn`
- HMAC-SHA256 signature in `X-Webhook-Signature` header
- Retry with exponential backoff (3 attempts)
- Delivery log with status, response code, latency

### 14.3 Custom Detection Rules

**Files:**
- `crates/pii_redacta_api/src/handlers/rules.rs` (new)
- `crates/pii_redacta_core/src/detection/custom_rules.rs` (new)
- New migration: `custom_rules` table

**Endpoints:**
```
POST   /api/v1/rules     → create custom rule
GET    /api/v1/rules     → list user's rules
PUT    /api/v1/rules/{id} → update rule
DELETE /api/v1/rules/{id} → delete rule
```

**Implementation:**
- Users define custom regex patterns with entity type labels
- Rules stored per-user in `custom_rules` table
- On detection: built-in patterns run first, then user's custom rules
- Validate regex at creation time (reject catastrophic backtracking)
- Check tier `custom_rules` feature flag

### Sprint 14 Summary

| Component | New Tests | Files Modified | Files Created |
|-----------|-----------|---------------|---------------|
| Batch Detection | 5 | lib.rs | handlers/batch.rs |
| Webhooks | 8 | lib.rs | handlers/webhooks.rs, jobs/webhook_delivery.rs, migration |
| Custom Rules | 6 | lib.rs, detection/ | handlers/rules.rs, detection/custom_rules.rs, migration |
| **Total** | **19** | **3** | **5+** |

**Target tests:** 332 total (313 + 19)

---

## File Map — All Sprints

### Sprint 11 (New Files)
```
crates/pii_redacta_api/src/jobs/mod.rs
crates/pii_redacta_api/src/jobs/processor.rs
crates/pii_redacta_core/src/db/redis.rs
```

### Sprint 12 (New Files)
```
crates/pii_redacta_core/migrations/NNNN_add_password_changed_at.sql
```

### Sprint 13 (New Files)
```
crates/pii_redacta_api/src/handlers/playground.rs
portal/src/pages/PlaygroundPage.tsx
```

### Sprint 14 (New Files)
```
crates/pii_redacta_api/src/handlers/batch.rs
crates/pii_redacta_api/src/handlers/webhooks.rs
crates/pii_redacta_api/src/handlers/rules.rs
crates/pii_redacta_api/src/jobs/webhook_delivery.rs
crates/pii_redacta_core/src/detection/custom_rules.rs
crates/pii_redacta_core/migrations/NNNN_create_webhook_endpoints.sql
crates/pii_redacta_core/migrations/NNNN_create_custom_rules.sql
```

---

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Job processor | Tokio background task | Simple, same process, no external deps |
| Redis integration | Optional (graceful degradation) | Dev/test works without Redis |
| Metrics | `metrics` + `metrics-exporter-prometheus` | Standard Rust ecosystem |
| Token blacklist | `password_changed_at` column + middleware check | No Redis dependency for security-critical path |
| Proxy trust | Configurable CIDR allowlist | Production-safe default (ignore X-Forwarded-For) |
| Playground processing | Synchronous (no job queue) | Small files, fast response, simpler UX |
| Webhook delivery | Fire-and-forget with retry | Non-blocking, eventually consistent |
| Custom rules | User-scoped regex with validation | Safe (backtracking check), per-user isolation |

---

## Verification Checklist (Per Sprint)

```bash
# Must pass before commit
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cd portal && npm run build
```

---

**Document Version:** 5.0
**Last Updated:** 2026-02-28
**Next Review:** After Sprint 11
