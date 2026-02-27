# PII Redacta Portal - Research & Recommendations

**Date:** 2026-02-27  
**Purpose:** Define portal requirements and recommend Rust-compatible technologies

---

## 1. Portal Feature Requirements

### Core Features for PII Service Portal

```
┌─────────────────────────────────────────────────────────────────┐
│                    PII REDACTA PORTAL                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │   Authentication            │  │   Billing   │             │
│  │  ├─ Login/Register          │  │  ├─ Usage   │             │
│  │  ├─ SSO (SAML/OAuth2)       │  │  ├─ Plans   │             │
│  │  ├─ MFA/2FA                 │  │  ├─ Invoices│             │
│  │  └─ Password Recovery       │  │  └─ Limits  │             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
│                                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │   API Management            │  │   Monitoring│             │
│  │  ├─ API Key Generation      │  │  ├─ Dashboard│             │
│  │  ├─ Key Rotation            │  │  ├─ Logs    │             │
│  │  ├─ Rate Limit Config       │  │  ├─ Alerts  │             │
│  │  ├─ IP Whitelisting         │  │  └─ Reports │             │
│  │  └─ Webhook Management      │  │             │             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
│                                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │   User Management           │  │   Security  │             │
│  │  ├─ Teams/Organizations     │  │  ├─ Audit Log│             │
│  │  ├─ Roles & Permissions     │  │  ├─ Compliance│            │
│  │  ├─ User Invitations        │  │  ├─ DPA/SLA │             │
│  │  └─ Profile Management      │  │  └─ Certifications│        │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. Database Recommendations

### Option 1: PostgreSQL (Recommended)

**Why PostgreSQL for PII Redacta:**
- ✅ Industry standard for enterprise applications
- ✅ Excellent Rust support via `sqlx` and `tokio-postgres`
- ✅ JSONB support for flexible metadata
- ✅ Row-level security (RLS) for multi-tenant isolation
- ✅ Full-text search capabilities
- ✅ ACID compliance for financial/billing data
- ✅ Encryption at rest (TDE)

**Rust Crates:**
```toml
[dependencies]
# Async PostgreSQL
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "uuid", "chrono"] }
# ORM alternative
diesel = { version = "2.1", features = ["postgres"] }
# Connection pooling
deadpool-postgres = "0.12"
```

**Schema Example:**
```sql
-- Users table with multi-tenant support
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    organization_id UUID REFERENCES organizations(id),
    role VARCHAR(50) NOT NULL DEFAULT 'member',
    mfa_enabled BOOLEAN DEFAULT FALSE,
    mfa_secret VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- API Keys table
CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    key_hash VARCHAR(255) NOT NULL,  -- Hashed key, never store plain
    key_prefix VARCHAR(8) NOT NULL,   -- First 8 chars for identification
    permissions JSONB DEFAULT '[]',   -- ["detect", "upload", "admin"]
    rate_limit_per_minute INTEGER DEFAULT 100,
    ip_whitelist JSONB,               -- ["192.168.1.0/24", "10.0.0.0/8"]
    expires_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    is_active BOOLEAN DEFAULT TRUE
);

-- Audit logs (immutable)
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    api_key_id UUID REFERENCES api_keys(id),
    action VARCHAR(100) NOT NULL,     -- "detection", "upload", "login"
    resource_type VARCHAR(50),        -- "file", "text", "api_key"
    resource_id UUID,
    ip_address INET,
    user_agent TEXT,
    metadata JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
) PARTITION BY RANGE (created_at);

-- Row Level Security for multi-tenancy
ALTER TABLE api_keys ENABLE ROW LEVEL SECURITY;
CREATE POLICY organization_isolation ON api_keys
    USING (user_id IN (
        SELECT id FROM users WHERE organization_id = current_setting('app.current_org')::UUID
    ));
```

### Option 2: SQLite (Development/Single-tenant)

**When to use:**
- Single-tenant deployments
- Development/testing
- Embedded systems
- Small deployments (< 1000 users)

**Rust Crates:**
```toml
sqlx = { version = "0.7", features = ["runtime-tokio", "sqlite"] }
rusqlite = { version = "0.30", features = ["bundled"] }
```

### Option 3: MongoDB (Document-heavy)

**When to use:**
- Highly variable schema
- Heavy metadata storage
- Preference for document model

**Rust Crates:**
```toml
mongodb = "2.8"
```

---

## 3. Authentication & User Management

### Recommendation: OAuth2 + OIDC with optional local auth

**Architecture:**
```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Client    │────▶│  PII Portal │────▶│   Identity  │
│  (Browser)  │     │   (Axum)    │     │  Provider   │
└─────────────┘     └─────────────┘     └─────────────┘
                           │
                           ▼
                    ┌─────────────┐
                    │  PostgreSQL │
                    │  (Sessions) │
                    └─────────────┘
```

**Recommended Approach: Hybrid**

### 3.1 Local Authentication (Default)

**Features:**
- Email/password with bcrypt/Argon2
- JWT access tokens + refresh tokens
- Email verification
- Password reset flow
- Rate limiting on auth endpoints

**Rust Crates:**
```toml
# Password hashing
argon2 = "0.5"  # Recommended over bcrypt

# JWT handling
jsonwebtoken = "9"

# Session management
tower-sessions = { version = "0.10", features = ["postgres-store"] }
```

### 3.2 OAuth2/OIDC Integration

**Supported Providers:**
- Google Workspace
- Microsoft Azure AD
- Okta
- Auth0
- Keycloak (self-hosted)

**Rust Crates:**
```toml
# OAuth2 client
oauth2 = "4.4"

# OpenID Connect
openidconnect = "3.5"

# Axum integration
axum-oauth2 = "0.1"
```

### 3.3 Multi-Factor Authentication (MFA)

**Options:**
- TOTP (Google Authenticator, Authy)
- WebAuthn/FIDO2 (YubiKey, Touch ID)
- SMS (not recommended for PII services)

**Rust Crates:**
```toml
# TOTP
totp-rs = { version = "5", features = ["qr"] }

# WebAuthn
webauthn-rs = "0.5"
```

---

## 4. API Key Management

### Recommended Implementation

**Key Format:**
```
pii_<prefix>_<random>
# Example: pii_live_abc123xyz789...
```

**Security Features:**
1. **Never store plaintext** - Only store SHA-256 hash
2. **Prefix for identification** - First 8 chars visible
3. **Rotation support** - Expiration dates, graceful rollover
4. **Scoped permissions** - Per-key permission grants
5. **IP restrictions** - Optional whitelist
6. **Usage quotas** - Per-minute rate limits

**Database Schema:**
```sql
CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    name VARCHAR(100) NOT NULL,
    
    -- Security: only store hash, never plaintext
    key_hash VARCHAR(255) NOT NULL,
    key_prefix VARCHAR(8) NOT NULL,
    
    -- Permissions as JSON array: ["detect:read", "upload:write", "admin"]
    permissions JSONB NOT NULL DEFAULT '["detect:read"]',
    
    -- Rate limiting
    rate_limit_per_minute INTEGER DEFAULT 100,
    rate_limit_per_hour INTEGER DEFAULT 1000,
    
    -- Network restrictions
    allowed_ips JSONB,  -- ["192.168.0.0/16", "10.0.0.0/8"]
    allowed_origins JSONB,  -- CORS origins for browser requests
    
    -- Lifecycle
    expires_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    use_count BIGINT DEFAULT 0,
    
    -- Status
    is_active BOOLEAN DEFAULT TRUE,
    revoked_at TIMESTAMPTZ,
    revoked_reason TEXT,
    
    -- Metadata
    created_at TIMESTAMPTZ DEFAULT NOW(),
    description TEXT
);

-- Index for fast key lookup by prefix
CREATE INDEX idx_api_keys_prefix ON api_keys(key_prefix);
```

**Key Generation (Rust):**
```rust
use rand::Rng;
use sha2::{Sha256, Digest};

pub struct ApiKey {
    pub plaintext: String,  // Show once to user
    pub prefix: String,
    pub hash: String,
}

impl ApiKey {
    pub fn generate() -> Self {
        let prefix = "pii_live";
        let random: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();
        
        let plaintext = format!("{}_{}", prefix, random);
        let hash = format!("{:x}", Sha256::digest(&plaintext));
        
        Self {
            prefix: prefix.to_string(),
            plaintext,
            hash,
        }
    }
}
```

---

## 5. Admin Portal UI Options

### Option 1: Leptos (Recommended for Rust)

**Why Leptos:**
- ✅ Full-stack Rust (no JavaScript needed)
- ✅ WASM frontend + Axum backend
- ✅ Reactive signals like React
- ✅ Server-side rendering (SSR) support
- ✅ Type-safe communication between frontend/backend

**Architecture:**
```rust
// Leptos component example
#[component]
fn ApiKeysPage() -> impl IntoView {
    let (keys, set_keys) = create_signal(vec![]);
    
    let load_keys = create_action(|_| async move {
        fetch_api_keys().await
    });
    
    view! {
        <div class="api-keys">
            <h1>"API Keys"</h1>
            <For
                each=keys
                key=|key| key.id
                children=|key| view! { <ApiKeyCard key=key /> }
            />
        </div>
    }
}
```

**Dependencies:**
```toml
leptos = { version = "0.6", features = ["nightly"] }
leptos_axum = "0.6"
leptos_router = "0.6"
```

### Option 2: React + TypeScript (Frontend) + Axum (Backend)

**When to use:**
- Team has React expertise
- Need rich ecosystem of UI components
- Complex dashboard requirements

**Rust Backend:**
```toml
# CORS for React dev server
tower-http = { version = "0.5", features = ["cors"] }
```

### Option 3: HTMX + Tera (Server-rendered)

**When to use:**
- Simplicity over interactivity
- SEO requirements
- Minimal JavaScript approach

**Rust Crates:**
```toml
tera = "1.19"
askama = "0.12"  # Type-safe templates
```

---

## 6. Recommended Tech Stack

```
┌─────────────────────────────────────────────────────────────┐
│                    RECOMMENDED STACK                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Frontend:     Leptos (WASM) + Tailwind CSS                 │
│                                                             │
│  Backend:      Axum + Tokio                                 │
│                                                             │
│  Database:     PostgreSQL + SQLx                            │
│                                                             │
│  Auth:         Argon2 + JWT + OAuth2 (optional)             │
│                                                             │
│  Sessions:     Redis (production) / In-memory (dev)         │
│                                                             │
│  Cache:        Redis (rate limiting, sessions)              │
│                                                             │
│  Search:       PostgreSQL Full-text (or Meilisearch)        │
│                                                             │
│  Queue:        PostgreSQL (MVP) / RabbitMQ (scale)          │
│                                                             │
│  Monitoring:   Prometheus + Grafana (existing)              │
│                                                             │
│  Logging:      tracing + tracing-subscriber                 │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 7. Implementation Phases

### Phase 1: Core Auth (Week 1-2)
- User registration/login
- JWT token handling
- Password reset flow
- Basic user profile

### Phase 2: API Keys (Week 3)
- Key generation
- Key hashing/storage
- Permission system
- Rate limiting per key

### Phase 3: Portal UI (Week 4-5)
- Dashboard with Leptos
- API key management page
- Usage statistics
- User settings

### Phase 4: Enterprise Features (Week 6-8)
- Team/organization support
- SSO integration (SAML/OIDC)
- Audit logs
- Advanced rate limiting

---

## 8. Security Considerations for Portal

```
┌─────────────────────────────────────────────────────────────┐
│                 SECURITY CHECKLIST                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  [ ] Argon2id for password hashing (NOT bcrypt)             │
│  [ ] HTTPS only (HSTS header)                               │
│  [ ] CSRF protection for web routes                         │
│  [ ] Rate limiting on auth endpoints                        │
│  [ ] Account lockout after failed attempts                  │
│  [ ] Secure session cookies (HttpOnly, Secure, SameSite)    │
│  [ ] API keys: hash only, never plaintext                   │
│  [ ] Input validation on all user inputs                    │
│  [ ] SQL injection prevention (use SQLx)                    │
│  [ ] XSS prevention (escape output)                         │
│  [ ] Content Security Policy headers                        │
│  [ ] Database encryption at rest                            │
│  [ ] Backup encryption                                      │
│  [ ] Audit logging for all sensitive operations             │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 9. Final Recommendation

**For PII Redacta Portal, I recommend:**

1. **Database:** PostgreSQL with SQLx (type-safe, async)
2. **Frontend:** Leptos (Rust WASM, full-stack)
3. **Auth:** Local auth (Argon2 + JWT) + optional OAuth2
4. **Sessions:** Redis in production
5. **API Keys:** SHA-256 hashed with prefix identification
6. **Rate Limiting:** Redis-based token bucket

**This gives you:**
- Single language (Rust) across entire stack
- Type safety from database to frontend
- Excellent performance
- Strong security posture
- Simplified deployment

---

**Next Steps:**
1. Review and approve tech stack
2. Design database schema for users/organizations
3. Implement authentication core
4. Build API key management
5. Create admin portal UI

---

**Document Version:** 1.0  
**Last Updated:** 2026-02-27
