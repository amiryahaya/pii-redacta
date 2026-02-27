# Authenticated Playground Implementation

**Status:** Ready for Development  
**Target:** Valid users only with limits & IP restrictions  
**Limits:** 10MB max, 5 submissions/day, IP tracking

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    AUTHENTICATED PLAYGROUND                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  User Flow:                                                     │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐     │
│  │ Register│───▶│  Login  │───▶│Playground│───▶│ Results │     │
│  │ (Email) │    │(Session)│    │(Limits)  │    │(History)│     │
│  └─────────┘    └─────────┘    └─────────┘    └─────────┘     │
│                                    │                            │
│                              ┌─────┴─────┐                      │
│                              │  Check:   │                      │
│                              │ - Auth?   │                      │
│                              │ - IP ban? │                      │
│                              │ - 5/day?  │                      │
│                              │ - 10MB?   │                      │
│                              └─────┬─────┘                      │
│                                    │                            │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Database Schema                                         │  │
│  │                                                          │  │
│  │  users ───────┬────── api_keys                           │  │
│  │    │          │           │                              │  │
│  │    └──────────┴────── usage_logs                         │  │
│  │                     (for billing/tiers)                  │  │
│  │                                                          │  │
│  │  ip_blocks (abuse prevention)                            │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Database Schema

### 1. Users Table

```sql
-- migrations/001_create_users.sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    
    -- Account status
    email_verified BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,
    is_admin BOOLEAN DEFAULT FALSE,
    
    -- Plan & billing (for future)
    plan VARCHAR(20) DEFAULT 'free',  -- 'free', 'starter', 'pro', 'enterprise'
    plan_expires_at TIMESTAMPTZ,
    
    -- Usage tracking (current period)
    current_period_start TIMESTAMPTZ DEFAULT NOW(),
    current_period_end TIMESTAMPTZ DEFAULT (NOW() + INTERVAL '30 days'),
    
    -- Metadata
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_login_at TIMESTAMPTZ,
    last_login_ip INET
);

-- Index for email lookups
CREATE INDEX idx_users_email ON users(email);
```

### 2. API Keys Table (Updated with HMAC)

```sql
-- migrations/002_create_api_keys.sql
CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Key identification
    name VARCHAR(100) NOT NULL,
    key_hash VARCHAR(255) NOT NULL,        -- HMAC-SHA256 hash
    key_identifier VARCHAR(8) NOT NULL,     -- First 8 chars for UI
    key_prefix VARCHAR(20) NOT NULL DEFAULT 'pii_live',
    key_version INTEGER DEFAULT 1,          -- For secret rotation
    
    -- Permissions
    permissions JSONB NOT NULL DEFAULT '["detect:read", "upload:write"]',
    allowed_ips JSONB,                      -- ["192.168.1.0/24"]
    allowed_origins JSONB,                  -- CORS origins
    
    -- Rate limiting
    rate_limit_per_minute INTEGER DEFAULT 100,
    rate_limit_per_hour INTEGER DEFAULT 1000,
    
    -- Usage tracking
    use_count BIGINT DEFAULT 0,
    last_used_at TIMESTAMPTZ,
    
    -- Lifecycle
    created_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT TRUE,
    revoked_at TIMESTAMPTZ,
    revoked_reason TEXT,
    
    UNIQUE(user_id, key_identifier)
);

CREATE INDEX idx_api_keys_user ON api_keys(user_id);
CREATE INDEX idx_api_keys_lookup ON api_keys(key_prefix, key_identifier);
```

### 3. Playground Usage Logs

```sql
-- migrations/003_create_playground_usage.sql
CREATE TABLE playground_submissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    
    -- Submission details
    submission_type VARCHAR(20) NOT NULL,  -- 'file' or 'text'
    filename VARCHAR(255),
    file_size_bytes INTEGER,
    mime_type VARCHAR(100),
    
    -- IP tracking for abuse prevention
    ip_address INET NOT NULL,
    ip_country VARCHAR(2),                  -- GeoIP country code
    user_agent TEXT,
    
    -- Results
    entities_found INTEGER DEFAULT 0,
    entity_types JSONB,                     -- ["email", "nric", "phone"]
    processing_time_ms INTEGER,
    was_redacted BOOLEAN DEFAULT FALSE,
    
    -- For replay/debugging (optional, 24h retention)
    input_hash VARCHAR(64),                 -- Hash of input (not content)
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Partition by month for performance
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

-- Create monthly partitions
CREATE TABLE playground_submissions_2024_03 
    PARTITION OF playground_submissions
    FOR VALUES FROM ('2024-03-01') TO ('2024-04-01');

-- Indexes for common queries
CREATE INDEX idx_playground_user_date ON playground_submissions(user_id, created_at);
CREATE INDEX idx_playground_ip_date ON playground_submissions(ip_address, created_at);
CREATE INDEX idx_playground_created ON playground_submissions(created_at);
```

### 4. IP Blocklist (Abuse Prevention)

```sql
-- migrations/004_create_ip_blocks.sql
CREATE TABLE ip_blocks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ip_address INET NOT NULL,
    ip_range CIDR,                          -- For blocking subnets
    
    -- Block reason
    reason VARCHAR(50) NOT NULL,            -- 'abuse', 'spam', 'brute_force'
    evidence TEXT,                          -- Details of abuse
    
    -- Automatic blocks
    failed_attempts INTEGER,                -- For auto-blocks
    
    -- Timestamps
    blocked_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,                 -- NULL = permanent
    created_by UUID REFERENCES users(id),   -- Admin who blocked
    
    UNIQUE(ip_address)
);

-- Index for fast lookups
CREATE INDEX idx_ip_blocks_address ON ip_blocks(ip_address);
CREATE INDEX idx_ip_blocks_expires ON ip_blocks(expires_at) WHERE expires_at IS NOT NULL;
```

### 5. User Sessions

```sql
-- migrations/005_create_sessions.sql
CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Session token (hashed)
    token_hash VARCHAR(255) NOT NULL,
    
    -- Metadata
    ip_address INET,
    user_agent TEXT,
    
    -- Lifecycle
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_used_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    
    -- Status
    is_valid BOOLEAN DEFAULT TRUE
);

CREATE INDEX idx_sessions_user ON user_sessions(user_id);
CREATE INDEX idx_sessions_token ON user_sessions(token_hash);
CREATE INDEX idx_sessions_expires ON user_sessions(expires_at);
```

---

## Limit Enforcement

### 1. Daily Limit Check (5 submissions/day)

```rust
use chrono::{Utc, Duration, NaiveDate};

pub struct PlaygroundLimits {
    pub submissions_today: i32,
    pub remaining_today: i32,
    pub max_file_size: usize,  // 10MB
    pub can_submit: bool,
    pub reset_at: DateTime<Utc>,
}

impl PlaygroundLimits {
    pub async fn check(pool: &PgPool, user_id: Uuid) -> Result<Self> {
        let today = Utc::now().date_naive();
        let tomorrow = today + Duration::days(1);
        
        // Count today's submissions
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) 
            FROM playground_submissions 
            WHERE user_id = $1 
            AND created_at >= $2 
            AND created_at < $3
            "#
        )
        .bind(user_id)
        .bind(today.and_hms_opt(0, 0, 0).unwrap())
        .bind(tomorrow.and_hms_opt(0, 0, 0).unwrap())
        .fetch_one(pool)
        .await?;
        
        let daily_limit = 5;
        let submissions_today = count as i32;
        let remaining_today = daily_limit - submissions_today;
        
        Ok(Self {
            submissions_today,
            remaining_today: remaining_today.max(0),
            max_file_size: 10 * 1024 * 1024,  // 10MB
            can_submit: submissions_today < daily_limit,
            reset_at: tomorrow.and_hms_opt(0, 0, 0).unwrap().and_utc(),
        })
    }
}
```

### 2. IP Restriction & Abuse Detection

```rust
pub struct IpChecker {
    pool: PgPool,
    redis: redis::aio::Connection,
}

impl IpChecker {
    /// Check if IP is blocked
    pub async fn is_blocked(&mut self, ip: IpAddr) -> Result<Option<String>> {
        // Check blocklist
        let block: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT reason FROM ip_blocks 
            WHERE (ip_address = $1 OR ip_range >>= $1)
            AND (expires_at IS NULL OR expires_at > NOW())
            "#
        )
        .bind(ip)
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some((reason,)) = block {
            return Ok(Some(reason));
        }
        
        // Check for auto-block (Redis-based)
        let key = format!("ip_failed:{}", ip);
        let failed: i32 = self.redis.get(&key).await.unwrap_or(0);
        
        if failed >= 10 {
            // Auto-block for 1 hour
            sqlx::query(
                "INSERT INTO ip_blocks (ip_address, reason, expires_at) VALUES ($1, 'brute_force', NOW() + INTERVAL '1 hour')"
            )
            .bind(ip)
            .execute(&self.pool)
            .await?;
            
            return Ok(Some("brute_force".to_string()));
        }
        
        Ok(None)
    }
    
    /// Record failed attempt (for auto-blocking)
    pub async fn record_failed(&mut self, ip: IpAddr) -> Result<()> {
        let key = format!("ip_failed:{}", ip);
        let count: i32 = self.redis.incr(&key, 1).await?;
        
        if count == 1 {
            // Set 1-hour expiry on first failure
            self.redis.expire(&key, 3600).await?;
        }
        
        Ok(())
    }
    
    /// Check for rapid submissions (rate limiting)
    pub async fn check_rate(&mut self, ip: IpAddr, user_id: Uuid) -> Result<bool> {
        // Per-IP rate limit (stricter than per-user)
        let ip_key = format!("playground_ip:{}", ip);
        let ip_count: i32 = self.redis.incr(&ip_key, 1).await?;
        
        if ip_count == 1 {
            self.redis.expire(&ip_key, 3600).await?; // 1 hour window
        }
        
        // Max 10 per IP per hour (regardless of user)
        if ip_count > 10 {
            return Ok(false);
        }
        
        Ok(true)
    }
}
```

### 3. Combined Middleware

```rust
use axum::{
    extract::{ConnectInfo, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::net::SocketAddr;

/// Playground protection middleware
pub async fn playground_limits<B>(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let ip = addr.ip();
    
    // 1. Check IP blocklist
    let mut ip_checker = IpChecker::new(state.pool.clone(), state.redis.clone());
    if let Some(reason) = ip_checker.is_blocked(ip).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        tracing::warn!("Blocked IP {} tried to access playground: {}", ip, reason);
        return Err(StatusCode::FORBIDDEN);
    }
    
    // 2. Check IP rate limit
    if !ip_checker.check_rate(ip, user_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    
    // 3. Get user from session
    let user = request.extensions()
        .get::<User>()
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // 4. Check user's daily limit
    let limits = PlaygroundLimits::check(&state.pool, user.id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if !limits.can_submit {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    
    // Add limits to request extensions for UI display
    request.extensions_mut().insert(limits);
    
    Ok(next.run(request).await)
}
```

---

## API Endpoints

### Authentication

```
POST /api/v1/auth/register
  Body: { "email": "...", "password": "..." }
  Response: { "user_id": "...", "message": "Check your email" }

POST /api/v1/auth/login
  Body: { "email": "...", "password": "..." }
  Response: { "session_token": "...", "expires_at": "..." }

POST /api/v1/auth/logout
  Headers: Cookie: session=...
  Response: 204 No Content

POST /api/v1/auth/forgot-password
  Body: { "email": "..." }
  Response: { "message": "Check your email" }
```

### Playground

```
GET /api/v1/playground/limits
  Headers: Cookie: session=...
  Response: {
    "submissions_today": 3,
    "remaining_today": 2,
    "max_file_size": 10485760,
    "can_submit": true,
    "reset_at": "2024-03-15T00:00:00Z"
  }

POST /api/v1/playground/submit
  Headers: Cookie: session=...
  Body: multipart/form-data (file)
  Limits:
    - File size: 10MB max
    - Daily count: 5 max
    - IP rate: 10/hour max
  Response: {
    "submission_id": "...",
    "entities": [...],
    "redacted_text": "...",
    "remaining_today": 1
  }

GET /api/v1/playground/history
  Headers: Cookie: session=...
  Query: ?limit=10&offset=0
  Response: {
    "submissions": [...],
    "total": 42
  }
```

### API Keys (User Management)

```
GET /api/v1/api-keys
  Response: [{ "id": "...", "name": "Production", "identifier": "abc123", ... }]

POST /api/v1/api-keys
  Body: { "name": "Staging", "permissions": ["detect:read"] }
  Response: { 
    "id": "...", 
    "key": "pii_live_abc123_xyz789...",  // ONLY SHOWN ONCE
    "identifier": "abc123"
  }

DELETE /api/v1/api-keys/:id
  Response: 204 No Content
```

---

## UI Mockup (Leptos)

```rust
// Playground Page Component
#[component]
fn PlaygroundPage() -> impl IntoView {
    let (limits, set_limits) = create_signal(None::<PlaygroundLimits>);
    let (result, set_result) = create_signal(None::<SubmissionResult>);
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(None::<String>);
    
    // Load limits on mount
    create_effect(move |_| {
        spawn_local(async move {
            match fetch_limits().await {
                Ok(l) => set_limits.set(Some(l)),
                Err(_) => set_error.set(Some("Failed to load limits".to_string())),
            }
        });
    });
    
    view! {
        <div class="playground-container">
            <h1>"PII Detection Playground"</h1>
            
            // Usage Meter
            {move || limits.get().map(|l| view! {
                <UsageMeter 
                    used=l.submissions_today 
                    limit=5 
                    reset_at=l.reset_at 
                />
            })}
            
            // Error display
            {move || error.get().map(|e| view! {
                <div class="error">{e}</div>
            })}
            
            // Upload Zone (disabled if at limit)
            {move || {
                let can_submit = limits.with(|l| l.as_ref().map(|l| l.can_submit).unwrap_or(false));
                view! {
                    <UploadZone 
                        on_submit=handle_submit
                        disabled=!can_submit || loading.get()
                        max_size=10 * 1024 * 1024
                    />
                }
            }}
            
            // Results
            {move || result.get().map(|r| view! {
                <ResultsDisplay result=r />
            })}
            
            // History link
            <a href="/history">"View Submission History"</a>
        </div>
    }
}

// Usage Meter Component
#[component]
fn UsageMeter(used: i32, limit: i32, reset_at: DateTime<Utc>) -> impl IntoView {
    let remaining = limit - used;
    let percentage = (used as f32 / limit as f32) * 100.0;
    
    view! {
        <div class="usage-meter">
            <div class="meter-header">
                <span>"Daily Usage"</span>
                <span class={if remaining == 0 { "at-limit" } else { "" }}>
                    {used} " / " {limit}
                </span>
            </div>
            <div class="meter-bar">
                <div 
                    class="meter-fill" 
                    style:width=format!("{}%", percentage)
                    class:warning=percentage > 80.0
                />
            </div>
            <div class="meter-footer">
                {remaining} " submissions remaining"
                <span class="reset-time">
                    "Resets " {format_time_until(reset_at)}
                </span>
            </div>
        </div>
    }
}
```

---

## Security Considerations

### 1. IP-Based Protections

```rust
// Auto-block IPs with suspicious patterns
pub async fn detect_abuse(pool: &PgPool, ip: IpAddr, user_id: Uuid) -> Result<bool> {
    // Pattern 1: Multiple accounts from same IP
    let account_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(DISTINCT user_id) 
        FROM user_sessions 
        WHERE ip_address = $1 
        AND created_at > NOW() - INTERVAL '24 hours'
        "#
    )
    .bind(ip)
    .fetch_one(pool)
    .await?;
    
    if account_count > 5 {
        // Flag for review or auto-block
        sqlx::query(
            "INSERT INTO ip_blocks (ip_address, reason, evidence) VALUES ($1, 'multi_account', $2)"
        )
        .bind(ip)
        .bind(format!("{} accounts in 24h", account_count))
        .execute(pool)
        .await?;
        
        return Ok(true);
    }
    
    // Pattern 2: Rapid submissions
    let recent_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM playground_submissions 
        WHERE ip_address = $1 AND created_at > NOW() - INTERVAL '1 minute'
        "#
    )
    .bind(ip)
    .fetch_one(pool)
    .await?;
    
    if recent_count > 10 {
        return Ok(true); // Abuse detected
    }
    
    Ok(false)
}
```

### 2. File Upload Security

```rust
pub async fn validate_upload(file: &MultipartFile) -> Result<(), UploadError> {
    // 1. Size check (10MB)
    if file.size() > 10 * 1024 * 1024 {
        return Err(UploadError::TooLarge);
    }
    
    // 2. MIME type validation
    let allowed = ["text/plain", "application/pdf"];
    if !allowed.contains(&file.mime_type()) {
        return Err(UploadError::InvalidType);
    }
    
    // 3. Magic bytes verification (prevent spoofing)
    let bytes = file.bytes().await?;
    if file.mime_type() == "application/pdf" && !bytes.starts_with(b"%PDF") {
        return Err(UploadError::InvalidContent);
    }
    
    // 4. Scan for malware (optional, ClamAV integration)
    // if malware_scan(&bytes).await? {
    //     return Err(UploadError::MalwareDetected);
    // }
    
    Ok(())
}
```

---

## Implementation Sprint Plan

### Sprint 9: Database & User Model (2 weeks)

**Week 1:**
- [ ] Set up PostgreSQL with SQLx
- [ ] Create user migrations
- [ ] User model with Argon2
- [ ] Registration endpoint

**Week 2:**
- [ ] Login/logout endpoints
- [ ] Session management
- [ ] Email verification (optional for MVP)
- [ ] Tests: 15

### Sprint 10: Limits & IP Protection (2 weeks)

**Week 1:**
- [ ] Playground submissions table
- [ ] Daily limit enforcement
- [ ] IP tracking
- [ ] IP blocklist

**Week 2:**
- [ ] Abuse detection (auto-block)
- [ ] Redis integration for rate limiting
- [ ] Limit status endpoint
- [ ] Tests: 15

### Sprint 11: API Key Management (2 weeks)

**Week 1:**
- [ ] API key table with HMAC
- [ ] Key generation (show once)
- [ ] Key listing/revocation

**Week 2:**
- [ ] API key authentication middleware
- [ ] Per-key rate limits
- [ ] Tests: 15

### Sprint 12: Playground UI (2 weeks)

**Week 1:**
- [ ] Set up Leptos
- [ ] Login page
- [ ] Upload component

**Week 2:**
- [ ] Results display
- [ ] Usage meter
- [ ] History page
- [ ] Tests: 15

**Total New Tests:** 60  
**Combined Total:** 150 tests

---

## Configuration

```yaml
# config/playground.yaml
playground:
  limits:
    daily_submissions: 5
    max_file_size: 10485760  # 10MB
    
  ip_protection:
    max_per_hour: 10
    max_failed_logins: 5
    auto_block_duration: "1h"
    
  abuse_detection:
    max_accounts_per_ip: 5
    max_submissions_per_minute: 10
    
  storage:
    history_retention: "30d"
    input_retention: "24h"  # For debugging only
```

---

## Ready to Start?

**Sprint 9 is ready:** Database foundation with user management

Key files to create:
1. `crates/pii_redacta_core/src/db/` - Database module
2. `crates/pii_redacta_api/src/auth/` - Authentication handlers
3. Migrations for users, api_keys, usage logs

**Shall I begin Sprint 9?**
