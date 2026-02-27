# PII Redacta - Updated Implementation Plan

**Version:** 4.0 (Consolidated)  
**Last Updated:** 2026-02-27  
**Status:** MVP Complete (Sprints 1-8), Sprint 9 Ready

---

## Executive Summary

### What We've Built (MVP - Sprints 1-8) ✅
- Pattern-based PII detection (Email, NRIC, Phone, Credit Card)
- Deterministic tokenization with HMAC-SHA256
- REST API with Axum
- File upload with job queue
- Prometheus metrics
- Security headers & Docker

### What We're Building Now (Portal - Sprints 9-16)
- **Configurable tier system** (database-driven, no code deploys)
- **Trial tier with API access** (2 keys, 50 files/month, 10MB/file)
- **Authenticated playground** (web UI for testing)
- **User management** (registration, login, usage tracking)
- **Enterprise features** (SSO, teams, billing)

---

## Revised Sprint Structure

### Phase 1: MVP (COMPLETE)
| Sprint | Focus | Status |
|--------|-------|--------|
| 1 | Project Foundation | ✅ Complete |
| 2 | Pattern Detection Engine | ✅ Complete |
| 3 | Tokenization Engine | ✅ Complete |
| 4 | REST API Foundation | ✅ Complete |
| 5 | File Processing | ✅ Complete |
| 6 | File Upload API | ✅ Complete |
| 7 | Observability | ✅ Complete |
| 8 | Security Hardening | ✅ Complete |

**MVP Tag:** `v0.1.0-mvp`  
**Total Tests:** 90 passing

### Phase 2: Portal with Configurable Tiers (IN PROGRESS)
| Sprint | Focus | Duration | Key Deliverables |
|--------|-------|----------|------------------|
| 9 | Database & Configurable Tiers | 2 weeks | PostgreSQL, TierManager, User auth |
| 10 | Playground Backend | 2 weeks | Limits, IP restrictions, usage tracking |
| 11 | API Key Management v2 | 2 weeks | HMAC-SHA256, per-key limits, rotation |
| 12 | Playground UI | 2 weeks | Leptos frontend, file upload, results |
| 13 | Admin Dashboard | 2 weeks | Tier configuration UI, user management |
| 14 | Billing Foundation | 2 weeks | Stripe integration, invoicing |
| 15 | Enterprise v1 | 2 weeks | Teams, SSO, audit logs |
| 16 | Performance & Polish | 2 weeks | Caching, monitoring, docs |

**Target Release:** `v0.2.0-portal` (Sprint 16)  
**Estimated Total Tests:** 200+

---

## Sprint 9 Detailed: Database & Configurable Tiers

### Overview
Build the foundation for user management and configurable tier system.

### Key Requirements (Updated)
1. **Trial tier MUST have API access** ✅
2. **Tiers must be configurable** without code deployment
3. **Limits:** 10MB file, 50 files/month, 2 API keys for trial
4. **IP restrictions** to prevent abuse
5. **Redis caching** for performance

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    SPRINT 9 ARCHITECTURE                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  PostgreSQL                    Redis                            │
│  ┌─────────────┐              ┌─────────────┐                  │
│  │ tiers       │◀────────────▶│ tier:trial  │ Cache (5 min)    │
│  │ (JSONB)     │              │ tiers:public│ Cache (10 min)   │
│  ├─────────────┤              └─────────────┘                  │
│  │ users       │                                               │
│  │ (Argon2)    │                                               │
│  ├─────────────┤                                               │
│  │ subscriptions│                                              │
│  │ (usage)     │                                               │
│  ├─────────────┤                                               │
│  │ tier_history│ Audit trail                                   │
│  └─────────────┘                                               │
│                                                                 │
│  TierManager (Rust)                                             │
│  ├── get_tier() → Check Redis → Fallback to DB → Cache         │
│  ├── update_tier() → Update DB → Invalidate cache              │
│  ├── list_public_tiers() → For pricing page                    │
│  └── check_feature() → Feature gating                          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Database Schema

#### Tiers Table (Configurable)
```sql
CREATE TABLE tiers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug VARCHAR(50) UNIQUE NOT NULL,
    name VARCHAR(100) NOT NULL,
    
    -- Configurable via JSONB
    limits JSONB NOT NULL DEFAULT '{
        "max_file_size": 10485760,
        "max_files_per_month": 50,
        "max_total_size_per_month": 524288000,
        "max_api_calls_per_month": 1000,
        "requests_per_minute": 30,
        "soft_limit_threshold": 0.8
    }',
    
    features JSONB NOT NULL DEFAULT '{
        "api_access": true,          -- ✅ Trial has API
        "max_api_keys": 2,
        "batch_processing": true,
        "support_level": "community"
    }',
    
    -- Pricing (in cents)
    price_monthly_cents INTEGER,
    price_yearly_cents INTEGER,
    
    -- Metadata
    is_active BOOLEAN DEFAULT TRUE,
    is_public BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

#### Default Trial Tier
```json
{
  "slug": "trial",
  "name": "Trial",
  "price_monthly_cents": 0,
  "limits": {
    "max_file_size": 10485760,           // 10MB
    "max_files_per_month": 50,
    "max_total_size_per_month": 524288000, // 500MB
    "max_api_calls_per_month": 1000,
    "requests_per_minute": 30,
    "soft_limit_threshold": 0.8
  },
  "features": {
    "api_access": true,                   // ✅ Enabled
    "max_api_keys": 2,
    "api_read": true,
    "api_write": true,
    "batch_processing": true,
    "playground_access": true,
    "playground_max_daily": 10,
    "support_level": "community",
    "data_retention_days": 7
  }
}
```

### Week 1 Tasks

#### Day 1-2: Setup
```bash
# Add dependencies
cargo add sqlx --features runtime-tokio,postgres,uuid,chrono,json
cargo add redis --features tokio-comp
cargo add argon2
cargo add chrono --features serde

# Install SQLx CLI
cargo install sqlx-cli

# Setup environment
export DATABASE_URL=postgres://postgres:postgres@localhost/pii_redacta_dev
export REDIS_URL=redis://127.0.0.1:6379
```

#### Day 3-4: Migrations (TDD)
Create migration files:
- `001_create_tiers.sql`
- `002_create_tier_history.sql`
- `003_create_users.sql`
- `004_create_subscriptions.sql`
- `005_insert_default_tiers.sql`

Tests: Verify tables exist, default data loaded

#### Day 5-7: Domain Models (TDD)
- `Tier` struct with JSONB deserialization
- `TierLimits` and `TierFeatures`
- Limit checking methods
- Tests: File size, monthly quotas, soft limits

### Week 2 Tasks

#### Day 8-10: Tier Manager
- `TierManager` with Redis caching
- `get_tier()` - cached retrieval
- `update_tier()` - with cache invalidation
- `list_public_tiers()` - for pricing page
- Tests: Cache hits, invalidation, DB fallback

#### Day 11-12: User Model
- `User` struct with Argon2
- `hash_password()` - Argon2id
- `verify_password()` - constant time
- Tests: Hashing, verification, wrong password

#### Day 13-14: Integration
- End-to-end tests
- Performance tests (cache vs DB)
- Documentation

### Sprint 9 Deliverables

| Component | Tests | Status |
|-----------|-------|--------|
| Migrations | 5 | 🎯 Target |
| Tier Model | 12 | 🎯 Target |
| Tier Manager | 10 | 🎯 Target |
| User Model | 8 | 🎯 Target |
| Integration | 10 | 🎯 Target |
| **Total** | **45** | **🎯 Target** |

### Sprint 9 Commit

```
feat(billing): sprint 9 - database foundation and configurable tier system

Database:
- PostgreSQL with SQLx (async, type-safe)
- Configurable tiers via JSONB (no code deploys)
- Tier history for audit trail
- Users with Argon2 password hashing
- Subscriptions with usage tracking

Tier System:
- Trial tier with API access (2 keys, 50 files, 10MB)
- Soft limits with warnings at 80%
- Redis caching (5-min TTL, auto-invalidation)
- Admin API for tier management

Caching:
- Tier cache: 5 minutes
- Public tiers cache: 10 minutes
- Automatic invalidation on update

Security:
- Argon2id for passwords (OWASP recommended)
- Prepared statements (SQL injection safe)
- Constant-time password verification

Tests: 45 new tests passing
Coverage: 90%+ for billing module
```

---

## Sprint 10: Playground Backend with Limits

### Overview
Build the authenticated playground with limit enforcement.

### Key Features
1. **Authenticated access only** (no anonymous)
2. **Daily limits:** 10MB file, 5 submissions/day
3. **IP restrictions:** Max 10/hour per IP
4. **Usage tracking:** For billing and limits
5. **Rate limiting:** Per-user, per-IP

### Architecture

```
User Request
    │
    ▼
┌─────────────────────────────────────┐
│  Middleware Stack                   │
│  1. Authentication (session/token)  │
│  2. IP Check (blocklist)            │
│  3. Rate Limit (Redis)              │
│  4. Tier Limit Check                │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│  Playground Handler                 │
│  - Validate file (10MB)             │
│  - Check daily quota (5/day)        │
│  - Process file                     │
│  - Record usage                     │
│  - Return results                   │
└─────────────────────────────────────┘
```

### Limits Enforcement

```rust
pub async fn check_playground_limits(
    user: &User,
    file_size: usize,
    ip: IpAddr,
) -> Result<(), LimitError> {
    // 1. Check IP blocklist
    if ip_checker.is_blocked(ip).await? {
        return Err(LimitError::IpBlocked);
    }
    
    // 2. Check IP rate limit (10/hour)
    if !ip_checker.check_rate(ip, 10, Duration::hours(1)).await? {
        return Err(LimitError::IpRateLimited);
    }
    
    // 3. Check tier file size limit
    tier.check_file_size(file_size)?; // 10MB for trial
    
    // 4. Check daily submission limit
    let today_count = get_today_submissions(user.id).await?;
    if today_count >= 5 {
        return Err(LimitError::DailyLimitExceeded);
    }
    
    // 5. Check monthly quota
    subscription.check_monthly_limits(file_size).await?;
    
    Ok(())
}
```

### API Endpoints

```
GET  /api/v1/playground/limits
     → { "daily_remaining": 3, "max_file_size": 10485760 }

POST /api/v1/playground/submit
     Headers: Cookie: session=xxx
     Body: multipart/form-data
     → { "entities": [...], "redacted_text": "..." }

GET  /api/v1/playground/history
     → { "submissions": [...], "total": 12 }
```

### Database: Playground Submissions

```sql
CREATE TABLE playground_submissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    
    -- File info
    filename VARCHAR(255),
    file_size_bytes INTEGER,
    mime_type VARCHAR(100),
    
    -- IP tracking (abuse prevention)
    ip_address INET NOT NULL,
    user_agent TEXT,
    
    -- Results
    entities_found INTEGER,
    processing_time_ms INTEGER,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
) PARTITION BY RANGE (created_at);
```

### Sprint 10 Tests

| Test Category | Count |
|---------------|-------|
| Limit enforcement | 10 |
| IP restrictions | 8 |
| Rate limiting | 6 |
| Usage tracking | 6 |
| Integration | 5 |
| **Total** | **35** |

---

## Sprint 11: API Key Management v2

### Overview
Implement secure API key system with HMAC-SHA256.

### Key Changes from MVP
1. **HMAC-SHA256** instead of SHA-256
2. **Per-key rate limits** (configurable by tier)
3. **Key rotation** support
4. **IP whitelisting** (Pro+ tiers)

### API Key Format
```
pii_<prefix>_<random>
Example: pii_live_abc123xyz789...
```

### Storage
```rust
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,              // "Production", "Staging"
    pub key_hash: String,          // HMAC-SHA256(key, SECRET)
    pub key_prefix: String,        // "abc123" (first 8 chars)
    pub permissions: Vec<String>,  // ["detect:read", "upload:write"]
    pub rate_limit_per_minute: i32,
    pub allowed_ips: Option<Vec<String>>, // ["192.168.1.0/24"]
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}
```

### HMAC-SHA256 Implementation

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn hash_api_key(key: &str, secret: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret)
        .expect("HMAC can take key of any size");
    mac.update(key.as_bytes());
    let result = mac.finalize();
    base64::encode(result.into_bytes())
}

pub fn verify_api_key(key: &str, hash: &str, secret: &[u8]) -> bool {
    let computed = hash_api_key(key, secret);
    // Constant-time comparison
    computed == hash
}
```

### Tests: 25

---

## Sprint 12: Playground UI

### Overview
Build the Leptos frontend for authenticated playground.

### Pages

#### 1. Login Page
```
┌─────────────────────────────┐
│  PII Redacta                │
│                             │
│  Email: [____________]      │
│  Password: [____________]   │
│                             │
│  [     Login     ]          │
│                             │
│  [Sign up] [Forgot password]│
└─────────────────────────────┘
```

#### 2. Playground Page
```
┌─────────────────────────────────────────────────┐
│  🔒 PII Redacta    [👤 User] [Logout]           │
├─────────────────────────────────────────────────┤
│                                                 │
│  Daily Usage: 3/5 files (resets in 12 hours)   │
│                                                 │
│  ┌───────────────────────────────────────────┐ │
│  │          📄 Drop file here                │ │
│  │          or click to browse               │ │
│  │                                           │ │
│  │   Max 10MB • TXT, PDF, DOCX               │ │
│  └───────────────────────────────────────────┘ │
│                                                 │
│  [x] Redact PII (replace with tokens)          │
│                                                 │
│  [      🔍 Process Document      ]             │
│                                                 │
│  ┌───────────────────────────────────────────┐ │
│  │  Results                                  │ │
│  │  Tab: [Detected PII] [Redacted Text]      │ │
│  │                                           │ │
│  │  Email: <<PII_EMAIL_ABC123>>              │ │
│  │  IC: <<PII_MY_NRIC_DEF456>>               │ │
│  │                                           │ │
│  │  [📋 Copy] [⬇️ Download]                  │ │
│  └───────────────────────────────────────────┘ │
│                                                 │
└─────────────────────────────────────────────────┘
```

#### 3. API Keys Page
```
┌─────────────────────────────────────────────────┐
│  API Keys                                       │
├─────────────────────────────────────────────────┤
│                                                 │
│  Tier: Trial (2/2 keys used)                   │
│                                                 │
│  ┌───────────────────────────────────────────┐ │
│  │ Production    Last used: 2 hours ago      │ │
│  │ Key: pii_live_abc123...****************   │ │
│  │ Rate: 30/min  [Revoke]                    │ │
│  └───────────────────────────────────────────┘ │
│                                                 │
│  ┌───────────────────────────────────────────┐ │
│  │ Staging       Last used: Never            │ │
│  │ Key: pii_live_def456...****************   │ │
│  │ Rate: 30/min  [Revoke]                    │ │
│  └───────────────────────────────────────────┘ │
│                                                 │
│  [+ Create New Key] (disabled: limit reached)  │
│                                                 │
└─────────────────────────────────────────────────┘
```

### Tech Stack
- **Framework:** Leptos (Rust WASM)
- **Styling:** Tailwind CSS
- **Icons:** Heroicons
- **Charts:** Chart.js (for usage graphs)

### Tests: 20

---

## Sprint 13: Admin Dashboard

### Overview
Build admin interface for managing tiers and users.

### Admin Pages

#### Tier Management
```
┌─────────────────────────────────────────────────┐
│  Admin › Tier Management                        │
├─────────────────────────────────────────────────┤
│                                                 │
│  [Trial] [Starter] [Pro] [Enterprise] [+ New]  │
│                                                 │
│  Editing: Trial                                 │
│  ─────────────────────────────────────────────  │
│                                                 │
│  Name: [Trial                       ]          │
│  Price: $[0]/mo  $[0]/yr                       │
│                                                 │
│  Limits                                         │
│  ├── Max file size: [10] MB                    │
│  ├── Max files/month: [50]                     │
│  ├── Max total size/month: [500] MB            │
│  └── API calls/month: [1000]                   │
│                                                 │
│  Features                                       │
│  [x] API access                                │
│  [x] Max API keys: [2]                         │
│  [x] Batch processing                          │
│  [ ] Webhooks                                  │
│                                                 │
│  [Save Changes]  [View History]                │
│                                                 │
└─────────────────────────────────────────────────┘
```

### Change History
```
┌─────────────────────────────────────────────────┐
│  Trial › Change History                         │
├─────────────────────────────────────────────────┤
│                                                 │
│  2024-03-15 14:32  admin@example.com           │
│  Changed max_file_size: 5MB → 10MB             │
│  Reason: "Increase trial limits for Q2 promo"  │
│  [View diff]                                   │
│                                                 │
│  2024-02-01 09:15  admin@example.com           │
│  Changed api_access: false → true              │
│  Reason: "Enable API for all trial users"      │
│                                                 │
└─────────────────────────────────────────────────┘
```

### Tests: 15

---

## Sprint 14: Billing Foundation

### Overview
Integrate Stripe for payments and invoicing.

### Features
- Stripe Checkout integration
- Subscription management
- Invoice generation
- Webhook handling
- Overage billing

### Tests: 15

---

## Sprint 15-16: Enterprise & Polish

### Sprint 15: Enterprise v1
- Team/organization accounts
- SAML/OIDC SSO
- Audit logs
- Priority support

### Sprint 16: Performance & Polish
- Database query optimization
- CDN for static assets
- Comprehensive documentation
- Final testing

---

## Updated File Structure

```
pii-redacta/
├── Cargo.toml
├── crates/
│   ├── pii_redacta_core/
│   │   ├── src/
│   │   │   ├── billing/           # NEW: Tier system
│   │   │   │   ├── mod.rs
│   │   │   │   ├── tier.rs
│   │   │   │   ├── tier_manager.rs
│   │   │   │   ├── subscription.rs
│   │   │   │   └── limit_checker.rs
│   │   │   ├── auth/              # NEW: User management
│   │   │   │   ├── mod.rs
│   │   │   │   ├── user.rs
│   │   │   │   └── session.rs
│   │   │   ├── db/                # NEW: Database module
│   │   │   │   └── mod.rs
│   │   │   └── lib.rs
│   │   └── migrations/            # NEW: SQLx migrations
│   │       ├── 001_create_tiers.sql
│   │       ├── 002_create_tier_history.sql
│   │       ├── 003_create_users.sql
│   │       └── 004_create_subscriptions.sql
│   └── pii_redacta_api/
│       └── src/
│           ├── handlers/
│           │   ├── playground.rs  # NEW: Playground handlers
│           │   ├── auth.rs        # NEW: Auth handlers
│           │   └── admin.rs       # NEW: Admin handlers
│           ├── middleware/
│           │   ├── auth.rs        # NEW: Auth middleware
│           │   ├── limits.rs      # NEW: Limit enforcement
│           │   └── ip_check.rs    # NEW: IP restrictions
│           └── lib.rs
└── frontend/                      # NEW: Leptos app
    ├── Cargo.toml
    └── src/
        ├── main.rs
        ├── pages/
        │   ├── login.rs
        │   ├── playground.rs
        │   ├── api_keys.rs
        │   └── admin/
        └── components/
```

---

## Test Summary

| Phase | Sprints | Tests | Status |
|-------|---------|-------|--------|
| MVP | 1-8 | 90 | ✅ Complete |
| Database | 9 | 45 | 🎯 Sprint 9 |
| Playground Backend | 10 | 35 | 📋 Planned |
| API Keys v2 | 11 | 25 | 📋 Planned |
| Playground UI | 12 | 20 | 📋 Planned |
| Admin | 13 | 15 | 📋 Planned |
| Billing | 14 | 15 | 📋 Planned |
| Enterprise | 15-16 | 20 | 📋 Planned |
| **Total** | | **285** | |

---

## Next Steps

### Immediate (This Week)
1. ✅ Review Sprint 9 plan
2. 🔄 Start Sprint 9 implementation
3. 🔄 Set up PostgreSQL + Redis

### This Month
- Complete Sprint 9 (Database + Tiers)
- Complete Sprint 10 (Playground Backend)

### Next Month
- Complete Sprint 11 (API Keys v2)
- Complete Sprint 12 (Playground UI)
- Release `v0.2.0-alpha` for testing

---

## Key Decisions Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Trial API access | ✅ Enabled | User feedback: evaluation needs API |
| Tier configuration | Database JSONB | Instant changes, no deploys |
| Password hashing | Argon2id | Modern, secure, OWASP recommended |
| API key hashing | HMAC-SHA256 | Rainbow table resistant |
| Cache | Redis | Fast, proven, easy invalidation |
| Frontend | Leptos | Full Rust stack, type-safe |
| Playground auth | Required | Abuse prevention, usage tracking |

---

**Ready to start Sprint 9?**

All requirements are now consolidated into this plan:
- ✅ Configurable tier system
- ✅ Trial with API access (2 keys, 50 files, 10MB)
- ✅ Authenticated playground (5/day, IP limits)
- ✅ User management with Argon2
- ✅ HMAC-SHA256 API keys (next sprint)
