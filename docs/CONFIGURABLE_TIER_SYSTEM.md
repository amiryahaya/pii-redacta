# Configurable Tier Management System

**Status:** Research Complete  
**Approach:** Database-Driven Configuration with Admin UI  
**Key Change:** Trial tier includes API access

---

## Research: How SaaS Companies Manage Tiers

### Pattern 1: Database-Driven (Recommended)

```
┌─────────────────────────────────────────────────────────────────┐
│  Database-Driven Tier Configuration                             │
│  (Used by: Stripe, Twilio, SendGrid, DigitalOcean)              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Admin Dashboard ──▶ Database ──▶ Application                    │
│       │                │              │                         │
│       ▼                ▼              ▼                         │
│  ┌─────────┐    ┌──────────┐    ┌──────────┐                   │
│  │ Edit    │───▶│ tiers    │───▶│ Runtime  │                   │
│  │ Tier    │    │ table    │    │ Check    │                   │
│  │ Properties   │          │    │          │                   │
│  └─────────┘    └──────────┘    └──────────┘                   │
│                                                                 │
│  Benefits:                                                      │
│  ✅ Change tiers without code deployment                        │
│  ✅ A/B testing different pricing                               │
│  ✅ Instant updates (no restart)                                │
│  ✅ Per-region pricing possible                                 │
│  ✅ Historical tracking of changes                              │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Pattern 2: Configuration File

```yaml
# config/tiers.yaml
# (Used by: Self-hosted apps, early-stage startups)

tiers:
  - slug: trial
    name: "Trial"
    price_monthly: 0
    limits:
      max_file_size: 10485760        # 10MB
      max_files_per_month: 50
      max_total_size_per_month: 524288000  # 500MB
    features:
      api_access: true               # ✅ Changed: Now enabled
      max_api_keys: 2                # Limited for trial
      support_level: "community"
```

**Comparison:**

| Aspect | Database-Driven | Config File |
|--------|-----------------|-------------|
| Update speed | Instant | Requires restart |
| A/B testing | ✅ Yes | ❌ No |
| Non-technical editing | ✅ Admin UI | ❌ YAML editing |
| Audit trail | ✅ Built-in | ❌ Git history only |
| Complexity | Higher | Lower |
| **Recommendation** | **✅ Use this** | For MVP only |

---

## Recommended Architecture: Hybrid Approach

### Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    CONFIGURABLE TIER SYSTEM                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Layer 1: Database (Source of Truth)                            │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  tiers table                                            │   │
│  │  ├── Core properties (price, limits, features)          │   │
│  │  ├── JSONB for flexibility                              │   │
│  │  └── Version history (track changes)                    │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│  Layer 2: Cache (Redis)      │                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  tier:trial ──▶ Cached config                           │   │
│  │  tier:starter ──▶ Cached config                         │   │
│  │  (TTL: 5 minutes, auto-invalidate on update)            │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│  Layer 3: Application        ▼                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  TierManager                                            │   │
│  │  ├── get_tier("trial") ──▶ Check cache ──▶ DB fallback │   │
│  │  ├── validate_limits(user_action)                       │   │
│  │  └── list_available_tiers()                             │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│  Layer 4: Admin UI           │                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  /admin/tiers                                           │   │
│  │  ├── View all tiers                                     │   │
│  │  ├── Edit properties (with validation)                  │   │
│  │  ├── Create new tier                                    │   │
│  │  └── View change history                                │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Updated Database Schema

### 1. Tiers Table (Enhanced)

```sql
-- migrations/006_create_tiers.sql
CREATE TABLE tiers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Identification
    slug VARCHAR(50) UNIQUE NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    
    -- Display
    display_name VARCHAR(100),
    short_description VARCHAR(255),
    features_list TEXT[],  -- ["10MB max file", "API access", "Email support"]
    is_recommended BOOLEAN DEFAULT FALSE,  -- Show "Most Popular" badge
    display_order INTEGER DEFAULT 0,
    
    -- Pricing (in cents for precision)
    price_monthly_cents INTEGER,
    price_yearly_cents INTEGER,
    setup_fee_cents INTEGER DEFAULT 0,
    currency VARCHAR(3) DEFAULT 'USD',
    
    -- Billing settings
    billing_intervals VARCHAR(10)[] DEFAULT ARRAY['monthly', 'yearly'],
    trial_days INTEGER DEFAULT 30,
    
    -- ============================================
    -- CONFIGURABLE LIMITS (JSONB)
    -- ============================================
    
    limits JSONB NOT NULL DEFAULT '{
        -- File upload limits
        "max_file_size": 10485760,
        "max_text_length": 100000,
        "allowed_file_types": ["txt", "pdf", "docx"],
        
        -- Periodic limits
        "max_files_per_month": 50,
        "max_total_size_per_month": 524288000,
        "max_api_calls_per_month": 1000,
        
        -- Rate limiting
        "requests_per_minute": 60,
        "requests_per_hour": 1000,
        "requests_per_day": 10000,
        "concurrent_requests": 2,
        
        -- Batch processing
        "max_batch_size": 1,
        "max_batch_total_size": 10485760,
        
        -- Soft limit warning threshold
        "soft_limit_threshold": 0.8
    }',
    
    -- ============================================
    -- CONFIGURABLE FEATURES (JSONB)
    -- ============================================
    
    features JSONB NOT NULL DEFAULT '{
        -- API access
        "api_access": true,
        "max_api_keys": 2,
        "api_read": true,
        "api_write": true,
        
        -- Processing
        "batch_processing": false,
        "priority_processing": false,
        "webhooks": false,
        "custom_rules": false,
        
        -- Support
        "support_level": "community",
        "support_response_hours": null,
        
        -- Data
        "data_retention_days": 7,
        "export_formats": ["json", "txt"],
        "audit_logs": false,
        
        -- Team
        "team_members": 1,
        "sso": false,
        "ip_whitelist": false,
        
        -- Playground
        "playground_access": true,
        "playground_max_daily": 5,
        "playground_history_days": 7
    }',
    
    -- ============================================
    -- OVERAGE CONFIGURATION
    -- ============================================
    
    overage_config JSONB DEFAULT '{
        "enabled": false,
        "price_per_file_cents": 0,
        "price_per_gb_cents": 0,
        "max_overage_amount_cents": 0,
        "notify_at_threshold": 0.9
    }',
    
    -- ============================================
    -- METADATA & STATUS
    -- ============================================
    
    is_active BOOLEAN DEFAULT TRUE,
    is_public BOOLEAN DEFAULT TRUE,
    is_deprecated BOOLEAN DEFAULT FALSE,  -- Hide from new signups
    deprecated_at TIMESTAMPTZ,
    deprecation_reason TEXT,
    
    -- Region-specific (for future internationalization)
    region VARCHAR(10) DEFAULT 'global',
    
    -- Versioning
    version INTEGER DEFAULT 1,
    created_by UUID REFERENCES users(id),
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_tiers_slug ON tiers(slug);
CREATE INDEX idx_tiers_active ON tiers(is_active, is_public);
CREATE INDEX idx_tiers_region ON tiers(region);

-- Trigger to track changes
CREATE TRIGGER tiers_updated_at
    BEFORE UPDATE ON tiers
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

### 2. Tier Change History (Audit Trail)

```sql
-- migrations/006b_create_tier_history.sql
CREATE TABLE tier_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tier_id UUID NOT NULL REFERENCES tiers(id),
    
    -- What changed
    changed_by UUID REFERENCES users(id),
    change_type VARCHAR(20) NOT NULL,  -- 'create', 'update', 'deprecate'
    
    -- Before/after values (for audit)
    previous_values JSONB,
    new_values JSONB,
    
    -- Reason for change
    change_reason TEXT,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Index for audit queries
CREATE INDEX idx_tier_history_tier ON tier_history(tier_id, created_at);
```

### 3. Default Tiers (MVP Configuration)

```sql
-- Insert default tiers with trial having API access
INSERT INTO tiers (
    slug, name, description, display_name, short_description,
    price_monthly_cents, price_yearly_cents, trial_days,
    limits, features, overage_config, display_order
) VALUES
(
    'trial', 'Trial', 'Free trial with full feature evaluation',
    'Trial', 'Try all features free for 30 days',
    0, 0, 30,
    '{
        "max_file_size": 10485760,
        "max_text_length": 100000,
        "allowed_file_types": ["txt", "pdf", "docx"],
        "max_files_per_month": 50,
        "max_total_size_per_month": 524288000,
        "max_api_calls_per_month": 1000,
        "requests_per_minute": 30,
        "requests_per_hour": 500,
        "requests_per_day": 5000,
        "concurrent_requests": 2,
        "max_batch_size": 5,
        "max_batch_total_size": 52428800,
        "soft_limit_threshold": 0.8
    }',
    '{
        "api_access": true,
        "max_api_keys": 2,
        "api_read": true,
        "api_write": true,
        "batch_processing": true,
        "priority_processing": false,
        "webhooks": false,
        "custom_rules": false,
        "support_level": "community",
        "support_response_hours": null,
        "data_retention_days": 7,
        "export_formats": ["json", "txt"],
        "audit_logs": false,
        "team_members": 1,
        "sso": false,
        "ip_whitelist": false,
        "playground_access": true,
        "playground_max_daily": 10,
        "playground_history_days": 7
    }',
    '{
        "enabled": false
    }',
    1
),
(
    'starter', 'Starter', 'For individual developers and small projects',
    'Starter', 'Perfect for getting started',
    4900, 49900, 14,
    '{
        "max_file_size": 52428800,
        "max_text_length": 500000,
        "allowed_file_types": ["txt", "pdf", "docx", "xlsx"],
        "max_files_per_month": 500,
        "max_total_size_per_month": 10737418240,
        "max_api_calls_per_month": 10000,
        "requests_per_minute": 100,
        "requests_per_hour": 3000,
        "requests_per_day": 30000,
        "concurrent_requests": 5,
        "max_batch_size": 20,
        "max_batch_total_size": 524288000,
        "soft_limit_threshold": 0.85
    }',
    '{
        "api_access": true,
        "max_api_keys": 5,
        "api_read": true,
        "api_write": true,
        "batch_processing": true,
        "priority_processing": false,
        "webhooks": true,
        "custom_rules": false,
        "support_level": "email",
        "support_response_hours": 48,
        "data_retention_days": 30,
        "export_formats": ["json", "txt", "csv", "pdf"],
        "audit_logs": false,
        "team_members": 3,
        "sso": false,
        "ip_whitelist": false,
        "playground_access": true,
        "playground_max_daily": 50,
        "playground_history_days": 30
    }',
    '{
        "enabled": true,
        "price_per_gb_cents": 500,
        "max_overage_amount_cents": 2500,
        "notify_at_threshold": 0.85
    }',
    2
),
(
    'professional', 'Professional', 'For growing teams with production workloads',
    'Professional', 'Scale with confidence',
    19900, 199900, 14,
    '{
        "max_file_size": 104857600,
        "max_text_length": 2000000,
        "allowed_file_types": ["txt", "pdf", "docx", "xlsx", "pptx"],
        "max_files_per_month": null,
        "max_total_size_per_month": 107374182400,
        "max_api_calls_per_month": 100000,
        "requests_per_minute": 500,
        "requests_per_hour": 15000,
        "requests_per_day": 150000,
        "concurrent_requests": 20,
        "max_batch_size": 100,
        "max_batch_total_size": 2147483648,
        "soft_limit_threshold": 0.9
    }',
    '{
        "api_access": true,
        "max_api_keys": 20,
        "api_read": true,
        "api_write": true,
        "batch_processing": true,
        "priority_processing": true,
        "webhooks": true,
        "custom_rules": true,
        "support_level": "priority",
        "support_response_hours": 24,
        "data_retention_days": 90,
        "export_formats": ["json", "txt", "csv", "pdf", "xml", "xlsx"],
        "audit_logs": true,
        "team_members": 10,
        "sso": false,
        "ip_whitelist": true,
        "playground_access": true,
        "playground_max_daily": null,
        "playground_history_days": 90
    }',
    '{
        "enabled": true,
        "price_per_gb_cents": 300,
        "max_overage_amount_cents": 15000,
        "notify_at_threshold": 0.9
    }',
    3
),
(
    'enterprise', 'Enterprise', 'Custom solutions for large organizations',
    'Enterprise', 'Dedicated support and custom terms',
    null, null, 30,
    '{
        "max_file_size": 1073741824,
        "max_text_length": null,
        "allowed_file_types": ["*"],
        "max_files_per_month": null,
        "max_total_size_per_month": null,
        "max_api_calls_per_month": null,
        "requests_per_minute": null,
        "requests_per_hour": null,
        "requests_per_day": null,
        "concurrent_requests": null,
        "max_batch_size": null,
        "max_batch_total_size": null,
        "soft_limit_threshold": 0.95
    }',
    '{
        "api_access": true,
        "max_api_keys": null,
        "api_read": true,
        "api_write": true,
        "batch_processing": true,
        "priority_processing": true,
        "webhooks": true,
        "custom_rules": true,
        "support_level": "dedicated",
        "support_response_hours": 4,
        "data_retention_days": null,
        "export_formats": ["json", "txt", "csv", "pdf", "xml", "xlsx", "custom"],
        "audit_logs": true,
        "team_members": null,
        "sso": true,
        "ip_whitelist": true,
        "playground_access": true,
        "playground_max_daily": null,
        "playground_history_days": null
    }',
    '{
        "enabled": false
    }',
    4
);
```

---

## Rust Implementation

### Tier Manager (Cached)

```rust
// crates/pii_redacta_core/src/billing/tier_manager.rs
use redis::AsyncCommands;
use serde_json::Value;

pub struct TierManager {
    pool: PgPool,
    redis: redis::aio::Connection,
}

impl TierManager {
    pub fn new(pool: PgPool, redis: redis::aio::Connection) -> Self {
        Self { pool, redis }
    }
    
    /// Get tier with caching
    pub async fn get_tier(&mut self, slug: &str) -> Result<Tier> {
        let cache_key = format!("tier:{}", slug);
        
        // Try cache first
        if let Ok(cached) = self.redis.get::<_, String>(&cache_key).await {
            if let Ok(tier) = serde_json::from_str::<Tier>(&cached) {
                return Ok(tier);
            }
        }
        
        // Fetch from database
        let tier = sqlx::query_as::<_, Tier>(
            "SELECT * FROM tiers WHERE slug = $1 AND is_active = true"
        )
        .bind(slug)
        .fetch_one(&self.pool)
        .await?;
        
        // Cache for 5 minutes
        let tier_json = serde_json::to_string(&tier)?;
        let _: () = self.redis.set_ex(&cache_key, tier_json, 300).await?;
        
        Ok(tier)
    }
    
    /// List all public tiers (for pricing page)
    pub async fn list_public_tiers(&self) -> Result<Vec<Tier>> {
        let tiers = sqlx::query_as::<_, Tier>(
            r#"
            SELECT * FROM tiers 
            WHERE is_active = true 
            AND is_public = true 
            AND is_deprecated = false
            ORDER BY display_order, price_monthly_cents ASC NULLS LAST
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(tiers)
    }
    
    /// Update tier (with cache invalidation)
    pub async fn update_tier(
        &mut self,
        slug: &str,
        updates: TierUpdate,
        changed_by: Uuid,
        reason: &str,
    ) -> Result<Tier> {
        // Get current tier for history
        let current = self.get_tier(slug).await?;
        
        // Build dynamic update query
        let mut query_parts = vec![];
        let mut params: Vec<&(dyn sqlx::Encode<'_, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>)> = vec![];
        
        if let Some(price) = updates.price_monthly_cents {
            query_parts.push("price_monthly_cents = $1");
            params.push(&price);
        }
        
        if let Some(limits) = &updates.limits {
            query_parts.push(format!("limits = limits || ${}::jsonb", params.len() + 1));
            params.push(&serde_json::to_value(limits)?);
        }
        
        if let Some(features) = &updates.features {
            query_parts.push(format!("features = features || ${}::jsonb", params.len() + 1));
            params.push(&serde_json::to_value(features)?);
        }
        
        // Add updated_at
        query_parts.push(format!("updated_at = ${}", params.len() + 1));
        params.push(&Utc::now());
        
        // Build and execute query
        let query = format!(
            "UPDATE tiers SET {} WHERE slug = ${} RETURNING *",
            query_parts.join(", "),
            params.len() + 1
        );
        
        let updated = sqlx::query_as::<_, Tier>(&query)
            .bind(slug)
            .fetch_one(&self.pool)
            .await?;
        
        // Record history
        sqlx::query(
            r#"
            INSERT INTO tier_history 
            (tier_id, changed_by, change_type, previous_values, new_values, change_reason)
            VALUES ($1, $2, 'update', $3, $4, $5)
            "#
        )
        .bind(updated.id)
        .bind(changed_by)
        .bind(serde_json::to_value(&current)?)
        .bind(serde_json::to_value(&updated)?)
        .bind(reason)
        .execute(&self.pool)
        .await?;
        
        // Invalidate cache
        let cache_key = format!("tier:{}", slug);
        let _: () = self.redis.del(&cache_key).await?;
        
        Ok(updated)
    }
    
    /// Check if a feature is enabled for a tier
    pub async fn check_feature(&mut self, tier_slug: &str, feature: &str) -> Result<bool> {
        let tier = self.get_tier(tier_slug).await?;
        
        // Navigate JSON path (e.g., "api_access" or "support.priority")
        let features = tier.features.0;
        
        Ok(match feature.split('.').collect::<Vec<_>>().as_slice() {
            [key] => features.get(key).and_then(|v| v.as_bool()).unwrap_or(false),
            [section, key] => features
                .get(section)
                .and_then(|s| s.get(key))
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            _ => false,
        })
    }
    
    /// Get effective limit (handles null/unlimited)
    pub async fn get_limit(&mut self, tier_slug: &str, limit_name: &str) -> Result<Option<usize>> {
        let tier = self.get_tier(tier_slug).await?;
        let limits = tier.limits.0;
        
        Ok(limits
            .get(limit_name)
            .and_then(|v| {
                if v.is_null() {
                    None  // Unlimited
                } else {
                    v.as_u64().map(|n| n as usize)
                }
            }))
    }
}

/// DTO for tier updates
#[derive(Debug, Default)]
pub struct TierUpdate {
    pub price_monthly_cents: Option<i32>,
    pub price_yearly_cents: Option<i32>,
    pub limits: Option<serde_json::Value>,
    pub features: Option<serde_json::Value>,
    pub is_active: Option<bool>,
}
```

### Tier-Aware Limit Checker

```rust
// crates/pii_redacta_core/src/billing/limit_checker.rs

pub struct LimitChecker {
    tier_manager: TierManager,
}

impl LimitChecker {
    /// Comprehensive limit check
    pub async fn check_action(
        &mut self,
        user: &User,
        subscription: &Subscription,
        action: UserAction,
    ) -> Result<LimitCheckResult> {
        let tier = self.tier_manager.get_tier(&subscription.tier_slug).await?;
        
        // Build context for limit checking
        let context = LimitContext {
            tier: &tier,
            subscription,
            action: &action,
        };
        
        // Check all relevant limits
        let mut results = vec![];
        
        match action {
            UserAction::UploadFile { size } => {
                results.push(self.check_file_size(&context, size).await?);
                results.push(self.check_monthly_files(&context).await?);
                results.push(self.check_monthly_size(&context, size).await?);
            }
            UserAction::ApiCall { endpoint, method } => {
                results.push(self.check_api_access(&context).await?);
                results.push(self.check_monthly_api_calls(&context).await?);
                results.push(self.check_rate_limits(&context).await?);
            }
            UserAction::BatchProcess { files, total_size } => {
                results.push(self.check_batch_allowed(&context).await?);
                results.push(self.check_batch_size(&context, files.len()).await?);
                results.push(self.check_batch_total_size(&context, total_size).await?);
            }
        }
        
        // Combine results
        let blocked = results.iter().any(|r| matches!(r, LimitResult::Exceeded(_)));
        let warnings: Vec<_> = results.iter()
            .filter_map(|r| match r {
                LimitResult::Approaching(w) => Some(w.clone()),
                _ => None,
            })
            .collect();
        
        if blocked {
            let exceeded = results.into_iter()
                .filter_map(|r| match r {
                    LimitResult::Exceeded(e) => Some(e),
                    _ => None,
                })
                .collect();
            return Ok(LimitCheckResult::Blocked(exceeded));
        }
        
        if !warnings.is_empty() {
            return Ok(LimitCheckResult::AllowedWithWarning(warnings));
        }
        
        Ok(LimitCheckResult::Allowed)
    }
    
    async fn check_file_size(&self, ctx: &LimitContext<'_>, size: usize) -> Result<LimitResult> {
        let max_size = ctx.tier.limits.max_file_size;
        
        if size > max_size {
            return Ok(LimitResult::Exceeded(LimitExceeded {
                resource: "file_size".to_string(),
                limit: max_size,
                actual: size,
                message: format!(
                    "File size {} exceeds maximum allowed {} for {} tier",
                    format_size(size),
                    format_size(max_size),
                    ctx.tier.name
                ),
                upgrade_url: Some("/upgrade".to_string()),
            }));
        }
        
        Ok(LimitResult::Ok)
    }
    
    async fn check_monthly_files(&self, ctx: &LimitContext<'_>) -> Result<LimitResult> {
        if let Some(max_files) = ctx.tier.limits.max_files_per_month {
            let used = ctx.subscription.files_used_this_period as usize;
            let threshold = (max_files as f64 * ctx.tier.limits.soft_limit_threshold) as usize;
            
            if used >= max_files {
                return Ok(LimitResult::Exceeded(LimitExceeded {
                    resource: "monthly_files".to_string(),
                    limit: max_files,
                    actual: used,
                    message: format!(
                        "You've used all {} files in your {} plan this month",
                        max_files, ctx.tier.name
                    ),
                    upgrade_url: Some("/upgrade".to_string()),
                }));
            }
            
            if used >= threshold {
                return Ok(LimitResult::Approaching(LimitWarning {
                    resource: "monthly_files".to_string(),
                    limit: max_files,
                    used,
                    remaining: max_files - used,
                    percentage: (used as f64 / max_files as f64) * 100.0,
                    message: format!(
                        "You've used {} of {} monthly files",
                        used, max_files
                    ),
                }));
            }
        }
        
        Ok(LimitResult::Ok)
    }
    
    async fn check_api_access(&self, ctx: &LimitContext<'_>) -> Result<LimitResult> {
        if !ctx.tier.features.api_access {
            return Ok(LimitResult::Exceeded(LimitExceeded {
                resource: "api_access".to_string(),
                limit: 0,
                actual: 1,
                message: "API access is not available on your current plan".to_string(),
                upgrade_url: Some("/upgrade".to_string()),
            }));
        }
        
        Ok(LimitResult::Ok)
    }
}

#[derive(Debug)]
pub enum LimitCheckResult {
    Allowed,
    AllowedWithWarning(Vec<LimitWarning>),
    Blocked(Vec<LimitExceeded>),
}

#[derive(Debug, Clone)]
pub struct LimitWarning {
    pub resource: String,
    pub limit: usize,
    pub used: usize,
    pub remaining: usize,
    pub percentage: f64,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct LimitExceeded {
    pub resource: String,
    pub limit: usize,
    pub actual: usize,
    pub message: String,
    pub upgrade_url: Option<String>,
}
```

---

## Admin API Endpoints

```rust
// Admin endpoints for managing tiers

// GET /api/v1/admin/tiers
// List all tiers with full configuration
async fn list_tiers_admin(State(state): State<AppState>) -> Result<Json<Vec<Tier>>, Error> {
    let tiers = sqlx::query_as::<_, Tier>("SELECT * FROM tiers ORDER BY display_order")
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(tiers))
}

// PATCH /api/v1/admin/tiers/:slug
// Update tier configuration
async fn update_tier_admin(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(updates): Json<TierUpdate>,
    AdminUser(admin): AdminUser,
) -> Result<Json<Tier>, Error> {
    let mut manager = TierManager::new(state.pool.clone(), state.redis.clone());
    
    let updated = manager
        .update_tier(&slug, updates, admin.id, "Admin configuration update")
        .await?;
    
    Ok(Json(updated))
}

// POST /api/v1/admin/tiers/:slug/clone
// Clone a tier as starting point for new tier
async fn clone_tier(
    State(state): State<AppState>,
    Path(source_slug): Path<String>,
    Json(new_tier): Json<NewTierRequest>,
) -> Result<Json<Tier>, Error> {
    let source = TierManager::new(state.pool.clone(), state.redis.clone())
        .get_tier(&source_slug)
        .await?;
    
    let cloned = sqlx::query_as::<_, Tier>(
        r#"
        INSERT INTO tiers 
        (slug, name, description, limits, features, overage_config, 
         price_monthly_cents, price_yearly_cents, display_order, created_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING *
        "#
    )
    .bind(&new_tier.slug)
    .bind(&new_tier.name)
    .bind(&new_tier.description)
    .bind(&source.limits)
    .bind(&source.features)
    .bind(&source.overage_config)
    .bind(new_tier.price_monthly_cents)
    .bind(new_tier.price_yearly_cents)
    .bind(new_tier.display_order)
    .bind(admin.id)
    .fetch_one(&state.pool)
    .await?;
    
    Ok(Json(cloned))
}
```

---

## Configuration Change Workflow

```
┌─────────────────────────────────────────────────────────────────┐
│              CHANGING TIER CONFIGURATION                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. Admin logs into /admin portal                               │
│     │                                                           │
│     ▼                                                           │
│  2. Navigate to "Tier Management"                               │
│     │                                                           │
│     ▼                                                           │
│  3. Select tier to edit (e.g., "trial")                         │
│     │                                                           │
│     ▼                                                           │
│  4. Modify properties:                                          │
│     • Change max_file_size: 10MB → 20MB                         │
│     • Change max_api_keys: 2 → 3                                │
│     • Change api_access: true (already enabled)                 │
│     │                                                           │
│     ▼                                                           │
│  5. Preview changes                                             │
│     • Show affected users: 1,234                                │
│     • Show estimated revenue impact                             │
│     │                                                           │
│     ▼                                                           │
│  6. Enter change reason: "Increase trial limits for Q2 promo"  │
│     │                                                           │
│     ▼                                                           │
│  7. Apply changes                                               │
│     • Update database                                           │
│     • Invalidate cache                                          │
│     • Record in audit log                                       │
│     • Takes effect immediately                                  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Updated Trial Tier Configuration

```yaml
# Current Trial Tier (MVP)
tier: trial
name: "Trial"
properties:
  # Pricing
  price_monthly: 0
  trial_days: 30
  
  # File limits
  max_file_size: 10485760          # 10MB
  max_files_per_month: 50
  max_total_size_per_month: 524288000  # 500MB
  
  # API limits  
  api_access: true                  # ✅ Enabled
  max_api_keys: 2                   # 2 API keys allowed
  max_api_calls_per_month: 1000
  requests_per_minute: 30
  
  # Playground
  playground_access: true
  playground_max_daily: 10          # Increased from 5
  
  # Features
  batch_processing: true            # Small batches allowed
  max_batch_size: 5
  support_level: "community"
  data_retention_days: 7
```

---

## Summary

| Aspect | Implementation |
|--------|----------------|
| **Configuration Store** | PostgreSQL JSONB columns |
| **Caching** | Redis (5-minute TTL) |
| **Admin Interface** | Web UI at /admin/tiers |
| **Audit Trail** | tier_history table |
| **Cache Invalidation** | Automatic on update |
| **Trial API Access** | ✅ Enabled with 2 keys |

**Key Benefits:**
- ✅ Change tiers without deployment
- ✅ A/B test pricing
- ✅ Instant updates globally
- ✅ Full audit trail
- ✅ Trial users get full API access

**Documents Updated:**
- `docs/CONFIGURABLE_TIER_SYSTEM.md` - Complete specification

**Ready to implement?**
