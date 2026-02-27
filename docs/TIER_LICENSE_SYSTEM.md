# Tier & License System

**Status:** MVP Design Complete  
**MVP Tier:** Trial (all users)  
**Future Tiers:** Starter, Professional, Enterprise  
**Billing Model:** Usage-based with soft/hard limits

---

## Industry Standard SaaS Tier Patterns

### Research: How Leading APIs Structure Tiers

```
┌─────────────────────────────────────────────────────────────────┐
│              INDUSTRY STANDARD PRACTICES                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Stripe (Payments)          OpenAI (AI/ML)                     │
│  ├── Free: $0               ├── Free: $5 credit               │
│  ├── Starter: 0.5% + $0.10  ├── Pay-as-you-go: per 1K tokens │
│  └── Scale: Custom          └── Enterprise: Custom commits    │
│                                                                 │
│  Twilio (SMS)               AWS (Cloud)                       │
│  ├── Pay-as-you-go          ├── Free tier: limited services   │
│  └── Volume discounts       └── Pay-as-you-go with tiers      │
│                                                                 │
│  Common Patterns:                                               │
│  1. Free/Trial tier for evaluation                            │
│  2. Soft limits with warnings before hard blocks               │
│  3. Usage-based pricing (per request, per MB)                 │
│  4. Monthly billing cycles with rollover options              │
│  5. Enterprise: Custom contracts with SLAs                    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Recommended Tier Structure

### Overview

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   TRIAL     │───▶│  STARTER    │───▶│    PRO      │───▶│ ENTERPRISE  │
│   (MVP)     │    │  ($49/mo)   │    │  ($199/mo)  │    │  (Custom)   │
├─────────────┤    ├─────────────┤    ├─────────────┤    ├─────────────┤
│ • 30 days   │    │ • 100MB/mo  │    │ • 1GB/mo    │    │ • Unlimited │
│ • 10MB/file │    │ • 50MB/file │    │ • 100MB/file│    │ • Custom    │
│ • 5 files   │    │ • 100 files │    │ • Unlimited │    │ • SLA       │
│ • No API    │    │ • API keys  │    │ • Priority  │    │ • Dedicated │
│ • Web only  │    │ • Email     │    │ • Support   │    │ • Support   │
│             │    │   support   │    │ • Webhooks  │    │ • SSO       │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
        │
        ▼
   All MVP users start here
   (can upgrade anytime)
```

---

## Detailed Tier Specifications

### Tier 1: Trial (MVP - All Users)

**Purpose:** Evaluation and testing  
**Duration:** 30 days (can be extended)  
**Price:** Free

```yaml
limits:
  # Per-transaction limits
  max_file_size: 10485760        # 10MB per file
  max_text_length: 100000        # 100K characters for text input
  
  # Periodic limits (monthly)
  max_files_per_month: 50        # 50 files total
  max_total_size_per_month: 524288000  # 500MB total per month
  max_api_calls_per_month: 100   # 100 API calls (if API access)
  
  # Rate limiting
  requests_per_minute: 10
  requests_per_hour: 60
  
  # Feature flags
  features:
    api_access: false            # Web playground only
    batch_processing: false      # Single file only
    webhooks: false
    custom_rules: false          # Standard patterns only
    priority_processing: false   # Standard queue
    support_level: "community"   # GitHub issues only
    data_retention_days: 7       # Results stored 7 days
    
  # Soft limits (warnings before hard stop)
  soft_limit_threshold: 0.8      # Warn at 80% of any limit
```

**Trial Expiry Behavior:**
- Option 1: Convert to Starter (auto-bill)
- Option 2: Downgrade to "Free" (limited features)
- Option 3: Data export then account suspension

---

### Tier 2: Starter (Post-MVP)

**Purpose:** Individual developers, small teams  
**Price:** $49/month or $499/year (15% discount)

```yaml
limits:
  # Per-transaction limits
  max_file_size: 52428800        # 50MB per file
  max_text_length: 500000        # 500K characters
  
  # Periodic limits (monthly)
  max_files_per_month: 500
  max_total_size_per_month: 10485760000  # 10GB per month
  max_api_calls_per_month: 5000
  
  # Rate limiting
  requests_per_minute: 60
  requests_per_hour: 1000
  concurrent_requests: 2
  
  # Feature flags
  features:
    api_access: true
    max_api_keys: 3
    batch_processing: false
    webhooks: false
    custom_rules: false
    priority_processing: false
    support_level: "email"       # Email support, 48h SLA
    data_retention_days: 30
    export_formats: ["json", "csv", "txt"]
    
overage:
  enabled: true
  price_per_gb: $5              # $5 per GB over limit
  max_overage: 5                # Max 5GB overage ($25)
```

---

### Tier 3: Professional (Post-MVP)

**Purpose:** Growing teams, production workloads  
**Price:** $199/month or $1999/year (16% discount)

```yaml
limits:
  # Per-transaction limits
  max_file_size: 104857600       # 100MB per file
  max_text_length: 2000000       # 2M characters
  
  # Periodic limits (monthly)
  max_files_per_month: unlimited
  max_total_size_per_month: 107374182400  # 100GB per month
  max_api_calls_per_month: 50000
  
  # Rate limiting
  requests_per_minute: 300
  requests_per_hour: 10000
  concurrent_requests: 10
  
  # Feature flags
  features:
    api_access: true
    max_api_keys: 10
    batch_processing: true       # Process multiple files
    max_batch_size: 100          # 100 files per batch
    webhooks: true
    custom_rules: true           # Custom PII patterns
    priority_processing: true    # Faster queue
    support_level: "priority"    # 24h SLA
    data_retention_days: 90
    export_formats: ["json", "csv", "txt", "pdf", "xml"]
    audit_logs: true
    team_members: 5              # 5 team members
    sso: false
    
overage:
  enabled: true
  price_per_gb: $3              # $3 per GB over limit
  max_overage: 50               # Max 50GB overage ($150)
  volume_discounts:
    - threshold: 500            # 500GB+ gets 10% off
      discount_percent: 10
    - threshold: 1000           # 1TB+ gets 20% off
      discount_percent: 20
```

---

### Tier 4: Enterprise (Post-MVP)

**Purpose:** Large organizations, compliance requirements  
**Price:** Custom (starts at $999/month)

```yaml
limits:
  # Per-transaction limits
  max_file_size: 1073741824      # 1GB per file (or custom)
  max_text_length: unlimited
  
  # Periodic limits (monthly)
  max_files_per_month: unlimited
  max_total_size_per_month: unlimited
  max_api_calls_per_month: unlimited
  
  # Rate limiting
  requests_per_minute: unlimited
  requests_per_hour: unlimited
  concurrent_requests: custom    # Dedicated resources
  
  # Feature flags
  features:
    api_access: true
    max_api_keys: unlimited
    batch_processing: true
    max_batch_size: custom
    webhooks: true
    custom_rules: true
    priority_processing: true
    dedicated_resources: true    # No shared queue
    support_level: "dedicated"   # Dedicated support engineer
    data_retention_days: custom  # Configurable
    export_formats: all
    audit_logs: true
    team_members: unlimited
    sso: true                    # SAML, OIDC
    sla: "99.99%"               # Uptime SLA
    private_cloud: optional      # On-premise deployment
    custom_models: true          # Train custom PII models
    
contract:
  min_commitment: 12            # 12-month minimum
  annual_discount: 20           # 20% off annual
  custom_terms: true
```

---

## Database Schema for Tier System

### 1. Tiers Table (Configuration)

```sql
-- migrations/006_create_tiers.sql
CREATE TABLE tiers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Basic info
    slug VARCHAR(50) UNIQUE NOT NULL,      -- 'trial', 'starter', 'pro', 'enterprise'
    name VARCHAR(100) NOT NULL,             -- 'Trial', 'Starter Plan'
    description TEXT,
    
    -- Pricing
    price_monthly_cents INTEGER,            -- In cents (e.g., 4900 = $49.00)
    price_yearly_cents INTEGER,             -- Annual price
    currency VARCHAR(3) DEFAULT 'USD',
    
    -- Limits (JSON for flexibility)
    limits JSONB NOT NULL DEFAULT '{
        "max_file_size": 10485760,
        "max_files_per_month": 50,
        "max_total_size_per_month": 524288000,
        "max_api_calls_per_month": 100,
        "requests_per_minute": 10,
        "requests_per_hour": 60
    }',
    
    -- Features (JSON)
    features JSONB NOT NULL DEFAULT '{
        "api_access": false,
        "batch_processing": false,
        "webhooks": false,
        "custom_rules": false,
        "priority_processing": false,
        "support_level": "community",
        "data_retention_days": 7
    }',
    
    -- Overage settings
    overage_config JSONB DEFAULT '{
        "enabled": false,
        "price_per_gb_cents": 0,
        "max_overage_gb": 0
    }',
    
    -- Status
    is_active BOOLEAN DEFAULT TRUE,
    is_public BOOLEAN DEFAULT TRUE,         -- Show on pricing page
    display_order INTEGER DEFAULT 0,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insert default tiers
INSERT INTO tiers (slug, name, price_monthly_cents, limits, features, display_order) VALUES
('trial', 'Trial', 0, 
 '{
    "max_file_size": 10485760,
    "max_text_length": 100000,
    "max_files_per_month": 50,
    "max_total_size_per_month": 524288000,
    "max_api_calls_per_month": 100,
    "requests_per_minute": 10,
    "requests_per_hour": 60,
    "soft_limit_threshold": 0.8
  }',
 '{
    "api_access": false,
    "batch_processing": false,
    "webhooks": false,
    "custom_rules": false,
    "priority_processing": false,
    "support_level": "community",
    "data_retention_days": 7,
    "max_api_keys": 0
  }',
  1
),
('starter', 'Starter', 4900,
 '{
    "max_file_size": 52428800,
    "max_text_length": 500000,
    "max_files_per_month": 500,
    "max_total_size_per_month": 10485760000,
    "max_api_calls_per_month": 5000,
    "requests_per_minute": 60,
    "requests_per_hour": 1000,
    "concurrent_requests": 2
  }',
 '{
    "api_access": true,
    "max_api_keys": 3,
    "batch_processing": false,
    "webhooks": false,
    "custom_rules": false,
    "priority_processing": false,
    "support_level": "email",
    "data_retention_days": 30,
    "export_formats": ["json", "csv", "txt"]
  }',
  2
),
('professional', 'Professional', 19900,
 '{
    "max_file_size": 104857600,
    "max_text_length": 2000000,
    "max_files_per_month": null,
    "max_total_size_per_month": 107374182400,
    "max_api_calls_per_month": 50000,
    "requests_per_minute": 300,
    "requests_per_hour": 10000,
    "concurrent_requests": 10
  }',
 '{
    "api_access": true,
    "max_api_keys": 10,
    "batch_processing": true,
    "max_batch_size": 100,
    "webhooks": true,
    "custom_rules": true,
    "priority_processing": true,
    "support_level": "priority",
    "data_retention_days": 90,
    "export_formats": ["json", "csv", "txt", "pdf", "xml"],
    "audit_logs": true,
    "team_members": 5
  }',
  3
),
('enterprise', 'Enterprise', null,
 '{
    "max_file_size": 1073741824,
    "max_text_length": null,
    "max_files_per_month": null,
    "max_total_size_per_month": null,
    "max_api_calls_per_month": null,
    "requests_per_minute": null,
    "requests_per_hour": null,
    "concurrent_requests": null
  }',
 '{
    "api_access": true,
    "max_api_keys": null,
    "batch_processing": true,
    "webhooks": true,
    "custom_rules": true,
    "priority_processing": true,
    "dedicated_resources": true,
    "support_level": "dedicated",
    "data_retention_days": null,
    "export_formats": ["json", "csv", "txt", "pdf", "xml", "custom"],
    "audit_logs": true,
    "team_members": null,
    "sso": true,
    "sla": "99.99%"
  }',
  4
);
```

### 2. User Subscriptions

```sql
-- migrations/007_create_subscriptions.sql
CREATE TABLE subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    tier_id UUID NOT NULL REFERENCES tiers(id),
    
    -- Billing
    status VARCHAR(20) NOT NULL DEFAULT 'trial',  -- 'trial', 'active', 'past_due', 'canceled', 'expired'
    billing_interval VARCHAR(10) DEFAULT 'monthly', -- 'monthly', 'yearly'
    
    -- Trial specific
    trial_started_at TIMESTAMPTZ,
    trial_ends_at TIMESTAMPTZ,
    trial_extended BOOLEAN DEFAULT FALSE,          -- Allow one extension
    
    -- Paid subscription
    current_period_started_at TIMESTAMPTZ,
    current_period_ends_at TIMESTAMPTZ,
    
    -- Usage tracking (resets each period)
    usage_reset_at TIMESTAMPTZ,
    files_used_this_period INTEGER DEFAULT 0,
    total_size_used_this_period BIGINT DEFAULT 0,   -- Bytes
    api_calls_used_this_period INTEGER DEFAULT 0,
    overage_charged_cents INTEGER DEFAULT 0,
    
    -- Grace period
    grace_period_ends_at TIMESTAMPTZ,              -- Allow usage after expiry
    
    -- Cancellation
    canceled_at TIMESTAMPTZ,
    cancellation_reason TEXT,
    
    -- Metadata
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(user_id)
);

-- Index for finding expired trials/subscriptions
CREATE INDEX idx_subscriptions_status ON subscriptions(status);
CREATE INDEX idx_subscriptions_trial_ends ON subscriptions(trial_ends_at) WHERE status = 'trial';
CREATE INDEX idx_subscriptions_period_ends ON subscriptions(current_period_ends_at) WHERE status = 'active';
```

### 3. Usage Tracking (Detailed)

```sql
-- migrations/008_create_usage_tracking.sql
CREATE TABLE usage_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    subscription_id UUID REFERENCES subscriptions(id),
    
    -- Event details
    event_type VARCHAR(50) NOT NULL,  -- 'file_upload', 'api_call', 'text_process'
    resource_type VARCHAR(50),        -- 'file', 'text', 'api'
    
    -- Usage amounts
    file_count INTEGER DEFAULT 1,
    total_bytes BIGINT,
    api_calls INTEGER DEFAULT 0,
    
    -- Cost tracking (for billable events)
    cost_cents INTEGER,               -- Calculated cost for this event
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Partition by month
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

-- Monthly partitions
CREATE TABLE usage_events_2024_03 
    PARTITION OF usage_events
    FOR VALUES FROM ('2024-03-01') TO ('2024-04-01');

-- Indexes
CREATE INDEX idx_usage_user_date ON usage_events(user_id, created_at);
CREATE INDEX idx_usage_subscription ON usage_events(subscription_id, created_at);
```

---

## Rust Implementation

### Tier Configuration

```rust
// crates/pii_redacta_core/src/billing/tier.rs
use serde::{Deserialize, Serialize};
use sqlx::types::Json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierLimits {
    pub max_file_size: usize,                    // Bytes per file
    pub max_text_length: Option<usize>,          // Characters
    pub max_files_per_month: Option<usize>,      // None = unlimited
    pub max_total_size_per_month: Option<usize>, // Bytes per month
    pub max_api_calls_per_month: Option<usize>,
    pub requests_per_minute: usize,
    pub requests_per_hour: usize,
    pub concurrent_requests: Option<usize>,
    pub soft_limit_threshold: f64,               // Warn at 80%
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierFeatures {
    pub api_access: bool,
    pub max_api_keys: Option<usize>,
    pub batch_processing: bool,
    pub max_batch_size: Option<usize>,
    pub webhooks: bool,
    pub custom_rules: bool,
    pub priority_processing: bool,
    pub dedicated_resources: bool,
    pub support_level: String,  // "community", "email", "priority", "dedicated"
    pub data_retention_days: usize,
    pub export_formats: Vec<String>,
    pub audit_logs: bool,
    pub team_members: Option<usize>,
    pub sso: bool,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Tier {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub price_monthly_cents: Option<i32>,
    pub price_yearly_cents: Option<i32>,
    pub limits: Json<TierLimits>,
    pub features: Json<TierFeatures>,
    pub is_active: bool,
}

impl Tier {
    /// Check if a file size is within tier limits
    pub fn check_file_size(&self, size: usize) -> Result<(), LimitError> {
        if size > self.limits.max_file_size {
            return Err(LimitError::FileTooLarge {
                max_allowed: self.limits.max_file_size,
                actual: size,
            });
        }
        Ok(())
    }
    
    /// Check if adding this usage would exceed monthly limits
    pub fn check_monthly_limits(
        &self,
        current_usage: &MonthlyUsage,
        new_file_count: usize,
        new_total_size: usize,
    ) -> Result<LimitStatus, LimitError> {
        let limits = &self.limits;
        
        // Check file count
        if let Some(max_files) = limits.max_files_per_month {
            let projected = current_usage.files + new_file_count;
            if projected > max_files {
                return Err(LimitError::MonthlyFileLimitExceeded {
                    limit: max_files,
                    current: current_usage.files,
                    attempted: new_file_count,
                });
            }
            
            // Soft limit warning at 80%
            let threshold = (max_files as f64 * limits.soft_limit_threshold) as usize;
            if projected > threshold && current_usage.files <= threshold {
                return Ok(LimitStatus::ApproachingLimit {
                    resource: "files",
                    used: projected,
                    limit: max_files,
                    percentage: (projected as f64 / max_files as f64) * 100.0,
                });
            }
        }
        
        // Check total size
        if let Some(max_size) = limits.max_total_size_per_month {
            let projected = current_usage.total_size + new_total_size;
            if projected > max_size {
                return Err(LimitError::MonthlySizeLimitExceeded {
                    limit: max_size,
                    current: current_usage.total_size,
                    attempted: new_total_size,
                });
            }
        }
        
        Ok(LimitStatus::WithinLimits)
    }
}

#[derive(Debug)]
pub enum LimitStatus {
    WithinLimits,
    ApproachingLimit {
        resource: String,
        used: usize,
        limit: usize,
        percentage: f64,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum LimitError {
    #[error("File size {actual} exceeds maximum allowed {max_allowed}")]
    FileTooLarge { max_allowed: usize, actual: usize },
    
    #[error("Monthly file limit exceeded: {current}/{limit}, attempted {attempted} more")]
    MonthlyFileLimitExceeded { limit: usize, current: usize, attempted: usize },
    
    #[error("Monthly size limit exceeded")]
    MonthlySizeLimitExceeded { limit: usize, current: usize, attempted: usize },
}

#[derive(Debug, Default)]
pub struct MonthlyUsage {
    pub files: usize,
    pub total_size: usize,
    pub api_calls: usize,
}
```

### Subscription Management

```rust
// crates/pii_redacta_core/src/billing/subscription.rs
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, sqlx::FromRow)]
pub struct Subscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tier_id: Uuid,
    pub status: SubscriptionStatus,
    pub billing_interval: BillingInterval,
    
    // Trial
    pub trial_started_at: Option<DateTime<Utc>>,
    pub trial_ends_at: Option<DateTime<Utc>>,
    pub trial_extended: bool,
    
    // Period
    pub current_period_started_at: Option<DateTime<Utc>>,
    pub current_period_ends_at: Option<DateTime<Utc>>,
    pub usage_reset_at: Option<DateTime<Utc>>,
    
    // Usage
    pub files_used_this_period: i32,
    pub total_size_used_this_period: i64,
    pub api_calls_used_this_period: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "subscription_status", rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Trial,
    Active,
    PastDue,
    Canceled,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "billing_interval", rename_all = "snake_case")]
pub enum BillingInterval {
    Monthly,
    Yearly,
}

impl Subscription {
    /// Create a new trial subscription for a user
    pub async fn create_trial(
        pool: &PgPool,
        user_id: Uuid,
        trial_days: i64,
    ) -> Result<Self> {
        let tier = sqlx::query_as::<_, Tier>(
            "SELECT * FROM tiers WHERE slug = 'trial'"
        )
        .fetch_one(pool)
        .await?;
        
        let now = Utc::now();
        let trial_ends = now + Duration::days(trial_days);
        
        let subscription = sqlx::query_as::<_, Subscription>(
            r#"
            INSERT INTO subscriptions 
            (user_id, tier_id, status, trial_started_at, trial_ends_at, usage_reset_at)
            VALUES ($1, $2, 'trial', $3, $4, $5)
            RETURNING *
            "#
        )
        .bind(user_id)
        .bind(tier.id)
        .bind(now)
        .bind(trial_ends)
        .bind(now)
        .fetch_one(pool)
        .await?;
        
        Ok(subscription)
    }
    
    /// Check if subscription is active (including trial)
    pub fn is_active(&self) -> bool {
        matches!(self.status, SubscriptionStatus::Trial | SubscriptionStatus::Active)
    }
    
    /// Check if trial has expired
    pub fn is_trial_expired(&self) -> bool {
        if self.status != SubscriptionStatus::Trial {
            return false;
        }
        
        match self.trial_ends_at {
            Some(ends_at) => Utc::now() > ends_at,
            None => false,
        }
    }
    
    /// Record usage and check limits
    pub async fn record_usage(
        &mut self,
        pool: &PgPool,
        tier: &Tier,
        file_count: usize,
        total_size: usize,
        api_calls: usize,
    ) -> Result<UsageResult> {
        // Check limits first
        let current = MonthlyUsage {
            files: self.files_used_this_period as usize,
            total_size: self.total_size_used_this_period as usize,
            api_calls: self.api_calls_used_this_period as usize,
        };
        
        let status = tier.check_monthly_limits(&current, file_count, total_size)?;
        
        // Update usage
        self.files_used_this_period += file_count as i32;
        self.total_size_used_this_period += total_size as i64;
        self.api_calls_used_this_period += api_calls as i32;
        
        sqlx::query(
            r#"
            UPDATE subscriptions 
            SET files_used_this_period = $1,
                total_size_used_this_period = $2,
                api_calls_used_this_period = $3
            WHERE id = $4
            "#
        )
        .bind(self.files_used_this_period)
        .bind(self.total_size_used_this_period)
        .bind(self.api_calls_used_this_period)
        .bind(self.id)
        .execute(pool)
        .await?;
        
        Ok(UsageResult {
            status,
            current_usage: MonthlyUsage {
                files: self.files_used_this_period as usize,
                total_size: self.total_size_used_this_period as usize,
                api_calls: self.api_calls_used_this_period as usize,
            },
        })
    }
    
    /// Reset usage for new billing period
    pub async fn reset_usage(&mut self, pool: &PgPool) -> Result<()> {
        self.files_used_this_period = 0;
        self.total_size_used_this_period = 0;
        self.api_calls_used_this_period = 0;
        self.usage_reset_at = Some(Utc::now());
        
        sqlx::query(
            r#"
            UPDATE subscriptions 
            SET files_used_this_period = 0,
                total_size_used_this_period = 0,
                api_calls_used_this_period = 0,
                usage_reset_at = NOW()
            WHERE id = $1
            "#
        )
        .bind(self.id)
        .execute(pool)
        .await?;
        
        Ok(())
    }
    
    /// Get remaining quota for display
    pub fn get_remaining_quota(&self, tier: &Tier) -> RemainingQuota {
        RemainingQuota {
            files: tier.limits.max_files_per_month.map(|limit| 
                limit.saturating_sub(self.files_used_this_period as usize)
            ),
            total_size: tier.limits.max_total_size_per_month.map(|limit|
                limit.saturating_sub(self.total_size_used_this_period as usize)
            ),
            api_calls: tier.limits.max_api_calls_per_month.map(|limit|
                limit.saturating_sub(self.api_calls_used_this_period as usize)
            ),
        }
    }
}

#[derive(Debug)]
pub struct UsageResult {
    pub status: LimitStatus,
    pub current_usage: MonthlyUsage,
}

#[derive(Debug)]
pub struct RemainingQuota {
    pub files: Option<usize>,
    pub total_size: Option<usize>,
    pub api_calls: Option<usize>,
}
```

---

## Middleware Integration

```rust
// crates/pii_redacta_api/src/middleware/billing.rs
use axum::{
    extract::{State, ConnectInfo},
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::net::SocketAddr;

/// Enforce tier limits middleware
pub async fn enforce_limits<B>(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let user = request.extensions()
        .get::<User>()
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Load subscription and tier
    let (subscription, tier) = sqlx::query_as::<_, (Subscription, Tier)>(
        r#"
        SELECT s.*, t.*
        FROM subscriptions s
        JOIN tiers t ON s.tier_id = t.id
        WHERE s.user_id = $1
        "#
    )
    .bind(user.id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Check if subscription is active
    if !subscription.is_active() {
        return Err(StatusCode::PAYMENT_REQUIRED); // 402
    }
    
    // Check trial expiry
    if subscription.is_trial_expired() {
        return Err(StatusCode::PAYMENT_REQUIRED);
    }
    
    // Check file size from content-length header
    if let Some(content_length) = request.headers().get("content-length") {
        if let Ok(length) = content_length.to_str().unwrap_or("0").parse::<usize>() {
            if let Err(e) = tier.check_file_size(length) {
                tracing::warn!("File size limit exceeded for user {}: {}", user.id, e);
                return Err(StatusCode::PAYLOAD_TOO_LARGE);
            }
        }
    }
    
    // Add subscription info to request for handlers
    request.extensions_mut().insert(subscription);
    request.extensions_mut().insert(tier);
    
    Ok(next.run(request).await)
}
```

---

## API Endpoints for Tiers

```
# Get current subscription status
GET /api/v1/subscription
Response: {
  "tier": {
    "slug": "trial",
    "name": "Trial",
    "limits": { "max_file_size": 10485760, ... }
  },
  "status": "trial",
  "trial_ends_at": "2024-04-15T00:00:00Z",
  "usage": {
    "files_used": 12,
    "files_limit": 50,
    "total_size_used": 125829120,
    "total_size_limit": 524288000,
    "reset_at": "2024-04-01T00:00:00Z"
  },
  "remaining": {
    "files": 38,
    "total_size": 398458880
  }
}

# Get available tiers (for upgrade page)
GET /api/v1/tiers
Response: [
  {
    "slug": "starter",
    "name": "Starter",
    "price_monthly": 4900,
    "price_yearly": 49900,
    "limits": { ... },
    "features": { ... }
  }
]

# Create checkout session (Stripe integration)
POST /api/v1/subscription/upgrade
Body: { "tier": "starter", "interval": "yearly" }
Response: { "checkout_url": "https://stripe.com/..." }

# Cancel subscription
POST /api/v1/subscription/cancel
Body: { "reason": "too_expensive", "feedback": "..." }
```

---

## UI Components

```rust
// Usage Meter with Tier Info
#[component]
fn SubscriptionStatus() -> impl IntoView {
    let (subscription, set_subscription) = create_signal(None::<SubscriptionView>);
    
    create_effect(move |_| {
        spawn_local(async move {
            let sub = fetch_subscription().await.unwrap();
            set_subscription.set(Some(sub));
        });
    });
    
    view! {
        {move || subscription.get().map(|s| view! {
            <div class="subscription-card">
                <div class="tier-header">
                    <h2>{s.tier.name}</h2>
                    <span class={format!("status {}", s.status)}>{s.status}</span>
                </div>
                
                // Trial countdown
                {if s.status == "trial" {
                    let days_left = s.trial_days_remaining;
                    view! {
                        <div class="trial-banner">
                            "Trial ends in " {days_left} " days. "
                            <a href="/upgrade">"Upgrade now"</a>
                        </div>
                    }
                } else {
                    view! { <></> }
                }}
                
                // Usage meters
                <UsageMeter 
                    label="Files"
                    used=s.usage.files_used
                    limit=s.limits.max_files_per_month
                />
                
                <UsageMeter 
                    label="Storage"
                    used=s.usage.total_size_used
                    limit=s.limits.max_total_size_per_month
                    formatter=|bytes| format!("{:.1} MB", bytes as f64 / 1048576.0)
                />
                
                // Upgrade button
                <a href="/upgrade" class="btn btn-primary">
                    "Upgrade Plan"
                </a>
            </div>
        })}
    }
}
```

---

## Summary

| Feature | Trial (MVP) | Starter | Pro | Enterprise |
|---------|-------------|---------|-----|------------|
| **Price** | Free | $49/mo | $199/mo | Custom |
| **Max File** | 10MB | 50MB | 100MB | 1GB+ |
| **Files/Month** | 50 | 500 | Unlimited | Unlimited |
| **Total Size/Month** | 500MB | 10GB | 100GB | Unlimited |
| **API Access** | ❌ | ✅ | ✅ | ✅ |
| **Support** | Community | Email | Priority | Dedicated |
| **Retention** | 7 days | 30 days | 90 days | Custom |

**All MVP users start on Trial tier** (30 days, then prompt to upgrade)

---

**Ready to implement the tier system with the playground?**
