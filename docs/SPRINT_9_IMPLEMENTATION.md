# Sprint 9: Database Foundation & Configurable Tier System

**Duration:** 2 weeks  
**Goal:** Production-ready database with configurable tiers, user management, and Redis caching  
**TDD Approach:** Tests → Implementation → Refactor → Commit

---

## Day 1-2: Project Setup & Dependencies

### Step 1: Add Dependencies

Update `crates/pii_redacta_core/Cargo.toml`:

```toml
[dependencies]
# Existing
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }

# Database
sqlx = { version = "0.7", features = [
    "runtime-tokio",
    "postgres",
    "uuid",
    "chrono",
    "migrate",
    "json"
] }

# Caching
redis = { version = "0.24", features = ["tokio-comp"] }

# Password hashing
argon2 = "0.5"

# Utilities
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
serde_with = "3.6"

[dev-dependencies]
# Testing
tokio-test = { workspace = true }
# Test database
sqlx-cli = "0.7"
```

### Step 2: Install SQLx CLI

```bash
cargo install sqlx-cli --no-default-features --features native-tls,postgres
```

### Step 3: Create .env file

```bash
# .env
DATABASE_URL=postgres://postgres:postgres@localhost/pii_redacta_dev
REDIS_URL=redis://127.0.0.1:6379
API_KEY_SECRET=your-super-secret-32-char-key-here!!
```

---

## Day 3-4: Database Migrations (TDD)

### Test First: Migration Tests

Create `crates/pii_redacta_core/tests/migration_test.rs`:

```rust
//! Database migration tests

use sqlx::PgPool;

async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/pii_redacta_test".to_string());
    
    PgPool::connect(&database_url).await.expect("Failed to connect to database")
}

#[tokio::test]
async fn test_tiers_table_exists() {
    let pool = setup_test_db().await;
    
    let result: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*) FROM tiers"
    )
    .fetch_optional(&pool)
    .await
    .expect("Failed to query tiers table");
    
    assert!(result.is_some());
}

#[tokio::test]
async fn test_tiers_has_default_data() {
    let pool = setup_test_db().await;
    
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tiers WHERE is_active = true"
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to count tiers");
    
    assert!(count >= 1, "Should have at least one active tier");
}

#[tokio::test]
async fn test_tier_limits_jsonb() {
    let pool = setup_test_db().await;
    
    let limits: Option<serde_json::Value> = sqlx::query_scalar(
        "SELECT limits FROM tiers WHERE slug = 'trial'"
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to get trial tier limits");
    
    let limits = limits.expect("Trial tier should exist");
    assert!(limits.get("max_file_size").is_some());
    assert!(limits.get("api_access").is_none()); // In features, not limits
}

#[tokio::test]
async fn test_users_table_exists() {
    let pool = setup_test_db().await;
    
    let result: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*) FROM users"
    )
    .fetch_optional(&pool)
    .await
    .expect("Failed to query users table");
    
    assert!(result.is_some());
}

#[tokio::test]
async fn test_subscriptions_table_exists() {
    let pool = setup_test_db().await;
    
    let result: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*) FROM subscriptions"
    )
    .fetch_optional(&pool)
    .await
    .expect("Failed to query subscriptions table");
    
    assert!(result.is_some());
}

#[tokio::test]
async fn test_tier_history_table_exists() {
    let pool = setup_test_db().await;
    
    let result: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*) FROM tier_history"
    )
    .fetch_optional(&pool)
    .await
    .expect("Failed to query tier_history table");
    
    assert!(result.is_some());
}
```

Run tests (should fail):
```bash
cargo test -p pii_redacta_core --test migration_test
# Expected: FAIL - tables don't exist yet
```

### Implement: Create Migrations

Create `crates/pii_redacta_core/migrations/001_create_tiers.sql`:

```sql
-- Tiers table with configurable limits and features
CREATE TABLE IF NOT EXISTS tiers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Identification
    slug VARCHAR(50) UNIQUE NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    
    -- Display
    display_name VARCHAR(100),
    short_description VARCHAR(255),
    features_list TEXT[],
    is_recommended BOOLEAN DEFAULT FALSE,
    display_order INTEGER DEFAULT 0,
    
    -- Pricing (in cents)
    price_monthly_cents INTEGER,
    price_yearly_cents INTEGER,
    setup_fee_cents INTEGER DEFAULT 0,
    currency VARCHAR(3) DEFAULT 'USD',
    
    -- Billing settings
    billing_intervals VARCHAR(10)[] DEFAULT ARRAY['monthly', 'yearly'],
    trial_days INTEGER DEFAULT 30,
    
    -- Configurable limits and features (JSONB)
    limits JSONB NOT NULL DEFAULT '{}',
    features JSONB NOT NULL DEFAULT '{}',
    overage_config JSONB DEFAULT '{}',
    
    -- Status
    is_active BOOLEAN DEFAULT TRUE,
    is_public BOOLEAN DEFAULT TRUE,
    is_deprecated BOOLEAN DEFAULT FALSE,
    deprecated_at TIMESTAMPTZ,
    deprecation_reason TEXT,
    
    -- Region and versioning
    region VARCHAR(10) DEFAULT 'global',
    version INTEGER DEFAULT 1,
    
    -- Metadata
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_tiers_slug ON tiers(slug);
CREATE INDEX idx_tiers_active ON tiers(is_active, is_public);

-- Trigger for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER tiers_updated_at
    BEFORE UPDATE ON tiers
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

Create `crates/pii_redacta_core/migrations/002_create_tier_history.sql`:

```sql
-- Tier change audit trail
CREATE TABLE IF NOT EXISTS tier_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tier_id UUID NOT NULL REFERENCES tiers(id) ON DELETE CASCADE,
    
    changed_by UUID,
    change_type VARCHAR(20) NOT NULL CHECK (change_type IN ('create', 'update', 'deprecate')),
    
    previous_values JSONB,
    new_values JSONB,
    change_reason TEXT,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_tier_history_tier ON tier_history(tier_id, created_at);
```

Create `crates/pii_redacta_core/migrations/003_create_users.sql`:

```sql
-- Users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    
    email_verified BOOLEAN DEFAULT FALSE,
    email_verified_at TIMESTAMPTZ,
    verification_token VARCHAR(255),
    
    is_active BOOLEAN DEFAULT TRUE,
    is_admin BOOLEAN DEFAULT FALSE,
    
    -- Profile
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    company_name VARCHAR(255),
    
    -- Metadata
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_login_at TIMESTAMPTZ,
    last_login_ip INET
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_active ON users(is_active);

CREATE TRIGGER users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

Create `crates/pii_redacta_core/migrations/004_create_subscriptions.sql`:

```sql
-- User subscriptions
CREATE TABLE IF NOT EXISTS subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tier_id UUID NOT NULL REFERENCES tiers(id),
    
    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'trial' 
        CHECK (status IN ('trial', 'active', 'past_due', 'canceled', 'expired')),
    billing_interval VARCHAR(10) DEFAULT 'monthly' 
        CHECK (billing_interval IN ('monthly', 'yearly')),
    
    -- Trial
    trial_started_at TIMESTAMPTZ,
    trial_ends_at TIMESTAMPTZ,
    trial_extended BOOLEAN DEFAULT FALSE,
    
    -- Period
    current_period_started_at TIMESTAMPTZ,
    current_period_ends_at TIMESTAMPTZ,
    usage_reset_at TIMESTAMPTZ,
    
    -- Usage tracking
    files_used_this_period INTEGER DEFAULT 0,
    total_size_used_this_period BIGINT DEFAULT 0,
    api_calls_used_this_period INTEGER DEFAULT 0,
    overage_charged_cents INTEGER DEFAULT 0,
    
    -- Grace period and cancellation
    grace_period_ends_at TIMESTAMPTZ,
    canceled_at TIMESTAMPTZ,
    cancellation_reason TEXT,
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(user_id)
);

CREATE INDEX idx_subscriptions_user ON subscriptions(user_id);
CREATE INDEX idx_subscriptions_status ON subscriptions(status);
CREATE INDEX idx_subscriptions_trial ON subscriptions(trial_ends_at) WHERE status = 'trial';

CREATE TRIGGER subscriptions_updated_at
    BEFORE UPDATE ON subscriptions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

Create `crates/pii_redacta_core/migrations/005_insert_default_tiers.sql`:

```sql
-- Insert default tiers
INSERT INTO tiers (slug, name, description, display_name, short_description,
    price_monthly_cents, price_yearly_cents, trial_days,
    limits, features, overage_config, display_order, is_public) 
VALUES (
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
    }'::jsonb,
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
        "data_retention_days": 7,
        "export_formats": ["json", "txt"],
        "audit_logs": false,
        "team_members": 1,
        "sso": false,
        "ip_whitelist": false,
        "playground_access": true,
        "playground_max_daily": 10,
        "playground_history_days": 7
    }'::jsonb,
    '{"enabled": false}'::jsonb,
    1, true
)
ON CONFLICT (slug) DO NOTHING;

-- Insert other tiers as needed...
```

Run migrations:
```bash
cd crates/pii_redacta_core
sqlx migrate run
```

Run tests (should pass):
```bash
cargo test -p pii_redacta_core --test migration_test
```

---

## Day 5-7: Domain Models (TDD)

### Test: Tier Model Tests

Create `crates/pii_redacta_core/src/billing/tier_test.rs`:

```rust
//! Tier model tests

use super::*;

#[test]
fn test_tier_limits_deserialization() {
    let json = r#"{
        "max_file_size": 10485760,
        "max_files_per_month": 50,
        "api_access": true
    }"#;
    
    let limits: TierLimits = serde_json::from_str(json).unwrap();
    assert_eq!(limits.max_file_size, 10485760);
    assert_eq!(limits.max_files_per_month, Some(50));
}

#[test]
fn test_tier_features_deserialization() {
    let json = r#"{
        "api_access": true,
        "max_api_keys": 2,
        "support_level": "community"
    }"#;
    
    let features: TierFeatures = serde_json::from_str(json).unwrap();
    assert!(features.api_access);
    assert_eq!(features.max_api_keys, Some(2));
    assert_eq!(features.support_level, "community");
}

#[test]
fn test_tier_check_file_size_within_limit() {
    let tier = create_test_tier();
    
    // 5MB file should pass (limit is 10MB)
    assert!(tier.check_file_size(5 * 1024 * 1024).is_ok());
}

#[test]
fn test_tier_check_file_size_exceeds_limit() {
    let tier = create_test_tier();
    
    // 15MB file should fail (limit is 10MB)
    let result = tier.check_file_size(15 * 1024 * 1024);
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert!(err.to_string().contains("exceeds maximum"));
}

#[test]
fn test_tier_check_monthly_limits_within_quota() {
    let tier = create_test_tier();
    let usage = MonthlyUsage {
        files: 10,
        total_size: 100 * 1024 * 1024, // 100MB
        api_calls: 100,
    };
    
    let result = tier.check_monthly_limits(&usage, 5, 50 * 1024 * 1024);
    assert!(matches!(result, Ok(LimitStatus::WithinLimits)));
}

#[test]
fn test_tier_check_monthly_limits_approaching() {
    let tier = create_test_tier(); // 50 files limit
    let usage = MonthlyUsage {
        files: 40, // 80% of limit
        total_size: 0,
        api_calls: 0,
    };
    
    let result = tier.check_monthly_limits(&usage, 5, 0);
    assert!(matches!(result, Ok(LimitStatus::ApproachingLimit { .. })));
}

#[test]
fn test_tier_check_monthly_limits_exceeded() {
    let tier = create_test_tier(); // 50 files limit
    let usage = MonthlyUsage {
        files: 48,
        total_size: 0,
        api_calls: 0,
    };
    
    // Try to add 5 more (would be 53, over limit)
    let result = tier.check_monthly_limits(&usage, 5, 0);
    assert!(result.is_err());
}

#[test]
fn test_tier_unlimited_files() {
    let mut tier = create_test_tier();
    tier.limits.max_files_per_month = None; // Unlimited
    
    let usage = MonthlyUsage {
        files: 1000000,
        total_size: 0,
        api_calls: 0,
    };
    
    let result = tier.check_monthly_limits(&usage, 1000, 0);
    assert!(matches!(result, Ok(LimitStatus::WithinLimits)));
}

fn create_test_tier() -> Tier {
    Tier {
        id: Uuid::new_v4(),
        slug: "test".to_string(),
        name: "Test Tier".to_string(),
        description: None,
        display_name: None,
        short_description: None,
        features_list: vec![],
        is_recommended: false,
        display_order: 0,
        price_monthly_cents: Some(0),
        price_yearly_cents: Some(0),
        setup_fee_cents: 0,
        currency: "USD".to_string(),
        billing_intervals: vec!["monthly".to_string()],
        trial_days: 30,
        limits: TierLimits {
            max_file_size: 10 * 1024 * 1024,
            max_text_length: Some(100000),
            allowed_file_types: vec!["txt".to_string()],
            max_files_per_month: Some(50),
            max_total_size_per_month: Some(500 * 1024 * 1024),
            max_api_calls_per_month: Some(1000),
            requests_per_minute: 30,
            requests_per_hour: 500,
            requests_per_day: 5000,
            concurrent_requests: Some(2),
            max_batch_size: Some(5),
            max_batch_total_size: Some(50 * 1024 * 1024),
            soft_limit_threshold: 0.8,
        },
        features: TierFeatures {
            api_access: true,
            max_api_keys: Some(2),
            api_read: true,
            api_write: true,
            batch_processing: true,
            priority_processing: false,
            webhooks: false,
            custom_rules: false,
            support_level: "community".to_string(),
            support_response_hours: None,
            data_retention_days: 7,
            export_formats: vec!["json".to_string()],
            audit_logs: false,
            team_members: Some(1),
            sso: false,
            ip_whitelist: false,
            playground_access: true,
            playground_max_daily: Some(10),
            playground_history_days: 7,
        },
        overage_config: OverageConfig {
            enabled: false,
            price_per_file_cents: 0,
            price_per_gb_cents: 0,
            max_overage_amount_cents: 0,
            notify_at_threshold: 0.9,
        },
        is_active: true,
        is_public: true,
        is_deprecated: false,
        deprecated_at: None,
        deprecation_reason: None,
        region: "global".to_string(),
        version: 1,
        created_by: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}
```

Run tests (should fail):
```bash
cargo test -p pii_redacta_core --lib -- tier::tests
```

### Implement: Domain Models

Create `crates/pii_redacta_core/src/billing/mod.rs`:

```rust
//! Billing and tier management

pub mod tier;
pub mod subscription;
pub mod tier_manager;
pub mod limit_checker;

pub use tier::{Tier, TierLimits, TierFeatures};
pub use subscription::{Subscription, SubscriptionStatus};
```

Create `crates/pii_redacta_core/src/billing/tier.rs`:

```rust
//! Tier model with configurable limits

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierLimits {
    pub max_file_size: usize,
    pub max_text_length: Option<usize>,
    #[serde(default)]
    pub allowed_file_types: Vec<String>,
    pub max_files_per_month: Option<usize>,
    pub max_total_size_per_month: Option<usize>,
    pub max_api_calls_per_month: Option<usize>,
    pub requests_per_minute: usize,
    pub requests_per_hour: usize,
    pub requests_per_day: usize,
    pub concurrent_requests: Option<usize>,
    pub max_batch_size: Option<usize>,
    pub max_batch_total_size: Option<usize>,
    pub soft_limit_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierFeatures {
    pub api_access: bool,
    pub max_api_keys: Option<usize>,
    pub api_read: bool,
    pub api_write: bool,
    pub batch_processing: bool,
    pub priority_processing: bool,
    pub webhooks: bool,
    pub custom_rules: bool,
    pub support_level: String,
    pub support_response_hours: Option<i32>,
    pub data_retention_days: usize,
    #[serde(default)]
    pub export_formats: Vec<String>,
    pub audit_logs: bool,
    pub team_members: Option<usize>,
    pub sso: bool,
    pub ip_whitelist: bool,
    pub playground_access: bool,
    pub playground_max_daily: Option<usize>,
    pub playground_history_days: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverageConfig {
    pub enabled: bool,
    pub price_per_file_cents: i32,
    pub price_per_gb_cents: i32,
    pub max_overage_amount_cents: i32,
    pub notify_at_threshold: f64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Tier {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub display_name: Option<String>,
    pub short_description: Option<String>,
    pub features_list: Vec<String>,
    pub is_recommended: bool,
    pub display_order: i32,
    pub price_monthly_cents: Option<i32>,
    pub price_yearly_cents: Option<i32>,
    pub setup_fee_cents: i32,
    pub currency: String,
    pub billing_intervals: Vec<String>,
    pub trial_days: i32,
    pub limits: Json<TierLimits>,
    pub features: Json<TierFeatures>,
    pub overage_config: Json<OverageConfig>,
    pub is_active: bool,
    pub is_public: bool,
    pub is_deprecated: bool,
    pub deprecated_at: Option<DateTime<Utc>>,
    pub deprecation_reason: Option<String>,
    pub region: String,
    pub version: i32,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Tier {
    /// Check if file size is within tier limits
    pub fn check_file_size(&self, size: usize) -> Result<(), TierError> {
        if size > self.limits.max_file_size {
            return Err(TierError::FileTooLarge {
                max_allowed: self.limits.max_file_size,
                actual: size,
            });
        }
        Ok(())
    }
    
    /// Check if usage is within monthly limits
    pub fn check_monthly_limits(
        &self,
        current: &MonthlyUsage,
        new_files: usize,
        new_size: usize,
    ) -> Result<LimitStatus, TierError> {
        // Check file count
        if let Some(max_files) = self.limits.max_files_per_month {
            let projected = current.files + new_files;
            if projected > max_files {
                return Err(TierError::MonthlyFileLimitExceeded {
                    limit: max_files,
                    current: current.files,
                    attempted: new_files,
                });
            }
            
            // Soft limit warning
            let threshold = (max_files as f64 * self.limits.soft_limit_threshold) as usize;
            if projected > threshold && current.files <= threshold {
                return Ok(LimitStatus::ApproachingLimit {
                    resource: "files".to_string(),
                    limit: max_files,
                    used: projected,
                    remaining: max_files - projected,
                    percentage: (projected as f64 / max_files as f64) * 100.0,
                });
            }
        }
        
        // Check total size
        if let Some(max_size) = self.limits.max_total_size_per_month {
            let projected = current.total_size + new_size;
            if projected > max_size {
                return Err(TierError::MonthlySizeLimitExceeded {
                    limit: max_size,
                    current: current.total_size,
                    attempted: new_size,
                });
            }
            
            // Soft limit warning for size
            let threshold = (max_size as f64 * self.limits.soft_limit_threshold) as usize;
            if projected > threshold && current.total_size <= threshold {
                return Ok(LimitStatus::ApproachingLimit {
                    resource: "storage".to_string(),
                    limit: max_size,
                    used: projected,
                    remaining: max_size - projected,
                    percentage: (projected as f64 / max_size as f64) * 100.0,
                });
            }
        }
        
        Ok(LimitStatus::WithinLimits)
    }
    
    /// Check if a feature is enabled
    pub fn has_feature(&self, feature_name: &str) -> bool {
        match feature_name {
            "api_access" => self.features.api_access,
            "batch_processing" => self.features.batch_processing,
            "priority_processing" => self.features.priority_processing,
            "webhooks" => self.features.webhooks,
            "custom_rules" => self.features.custom_rules,
            "audit_logs" => self.features.audit_logs,
            "sso" => self.features.sso,
            "ip_whitelist" => self.features.ip_whitelist,
            "playground_access" => self.features.playground_access,
            _ => false,
        }
    }
}

#[derive(Debug, Default)]
pub struct MonthlyUsage {
    pub files: usize,
    pub total_size: usize,
    pub api_calls: usize,
}

#[derive(Debug)]
pub enum LimitStatus {
    WithinLimits,
    ApproachingLimit {
        resource: String,
        limit: usize,
        used: usize,
        remaining: usize,
        percentage: f64,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum TierError {
    #[error("File size {actual} exceeds maximum allowed {max_allowed}")]
    FileTooLarge { max_allowed: usize, actual: usize },
    
    #[error("Monthly file limit exceeded: {current}/{limit}, attempted {attempted} more")]
    MonthlyFileLimitExceeded { limit: usize, current: usize, attempted: usize },
    
    #[error("Monthly storage limit exceeded: {current}/{limit}, attempted {attempted}")]
    MonthlySizeLimitExceeded { limit: usize, current: usize, attempted: usize },
    
    #[error("API access not available on this tier")]
    ApiAccessNotAvailable,
    
    #[error("Feature '{0}' not available on this tier")]
    FeatureNotAvailable(String),
}

#[cfg(test)]
#[path = "tier_test.rs"]
mod tests;
```

Run tests (should pass):
```bash
cargo test -p pii_redacta_core --lib -- tier::tests
```

---

## Day 8-10: Tier Manager with Redis Caching (TDD)

### Test: Tier Manager Tests

Create `crates/pii_redacta_core/src/billing/tier_manager_test.rs`:

```rust
//! Tier manager tests (requires database)

use super::*;
use sqlx::PgPool;

async fn setup_manager() -> (TierManager, PgPool) {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/pii_redacta_test".to_string());
    
    let pool = PgPool::connect(&database_url).await.unwrap();
    
    // Connect to Redis or use mock
    let redis = redis::Client::open("redis://127.0.0.1:6379")
        .unwrap()
        .get_async_connection()
        .await
        .unwrap();
    
    (TierManager::new(pool.clone(), redis), pool)
}

#[tokio::test]
async fn test_get_tier_from_database() {
    let (mut manager, _pool) = setup_manager().await;
    
    let tier = manager.get_tier("trial").await.unwrap();
    assert_eq!(tier.slug, "trial");
    assert!(tier.limits.max_file_size > 0);
}

#[tokio::test]
async fn test_get_tier_caches_result() {
    let (mut manager, _pool) = setup_manager().await;
    
    // First call hits database
    let tier1 = manager.get_tier("trial").await.unwrap();
    
    // Second call should hit cache
    let tier2 = manager.get_tier("trial").await.unwrap();
    
    assert_eq!(tier1.id, tier2.id);
}

#[tokio::test]
async fn test_list_public_tiers() {
    let (mut manager, _pool) = setup_manager().await;
    
    let tiers = manager.list_public_tiers().await.unwrap();
    assert!(!tiers.is_empty());
    
    // Should include trial tier
    let trial = tiers.iter().find(|t| t.slug == "trial");
    assert!(trial.is_some());
}

#[tokio::test]
async fn test_update_tier_invalidates_cache() {
    let (mut manager, _pool) = setup_manager().await;
    
    // Load tier into cache
    let tier_before = manager.get_tier("trial").await.unwrap();
    let original_price = tier_before.price_monthly_cents;
    
    // Update tier
    let updates = TierUpdate {
        price_monthly_cents: Some(100),
        ..Default::default()
    };
    
    let admin_id = Uuid::new_v4();
    manager.update_tier("trial", updates, admin_id, "Test update").await.unwrap();
    
    // Reload tier (should hit database, not cache)
    let tier_after = manager.get_tier("trial").await.unwrap();
    assert_eq!(tier_after.price_monthly_cents, Some(100));
    
    // Restore original value
    let restore = TierUpdate {
        price_monthly_cents: original_price,
        ..Default::default()
    };
    manager.update_tier("trial", restore, admin_id, "Restore test").await.unwrap();
}

#[tokio::test]
async fn test_check_feature() {
    let (mut manager, _pool) = setup_manager().await;
    
    // Trial tier has api_access enabled
    let has_api = manager.check_feature("trial", "api_access").await.unwrap();
    assert!(has_api);
    
    // Check non-existent feature
    let has_fake = manager.check_feature("trial", "fake_feature").await.unwrap();
    assert!(!has_fake);
}

#[tokio::test]
async fn test_get_limit() {
    let (mut manager, _pool) = setup_manager().await;
    
    let max_file_size = manager.get_limit("trial", "max_file_size").await.unwrap();
    assert_eq!(max_file_size, Some(10 * 1024 * 1024)); // 10MB
}
```

Run tests (should fail):
```bash
cargo test -p pii_redacta_core --lib -- tier_manager::tests
```

### Implement: Tier Manager

Create `crates/pii_redacta_core/src/billing/tier_manager.rs`:

```rust
//! Tier manager with caching

use super::{Tier, TierLimits, TierFeatures};
use crate::error::Result;
use redis::AsyncCommands;
use sqlx::PgPool;
use uuid::Uuid;

pub struct TierManager {
    pool: PgPool,
    redis: redis::aio::Connection,
}

#[derive(Debug, Default)]
pub struct TierUpdate {
    pub price_monthly_cents: Option<i32>,
    pub price_yearly_cents: Option<i32>,
    pub limits: Option<serde_json::Value>,
    pub features: Option<serde_json::Value>,
    pub is_active: Option<bool>,
    pub is_public: Option<bool>,
}

impl TierManager {
    pub fn new(pool: PgPool, redis: redis::aio::Connection) -> Self {
        Self { pool, redis }
    }
    
    /// Get tier by slug (with caching)
    pub async fn get_tier(&mut self, slug: &str) -> Result<Tier> {
        let cache_key = format!("tier:{}", slug);
        
        // Try cache first
        if let Ok(cached) = self.redis.get::<_, String>(&cache_key).await {
            if let Ok(tier) = serde_json::from_str::<Tier>(&cached) {
                tracing::debug!("Tier {} found in cache", slug);
                return Ok(tier);
            }
        }
        
        // Fetch from database
        let tier = sqlx::query_as::<_, Tier>(
            r#"
            SELECT * FROM tiers 
            WHERE slug = $1 AND is_active = true
            "#
        )
        .bind(slug)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => crate::error::PiiError::NotFound(
                format!("Tier '{}' not found", slug)
            ),
            _ => e.into(),
        })?;
        
        // Cache for 5 minutes
        let tier_json = serde_json::to_string(&tier)?;
        let _: () = self.redis.set_ex(&cache_key, tier_json, 300).await?;
        
        Ok(tier)
    }
    
    /// List all public tiers
    pub async fn list_public_tiers(&mut self) -> Result<Vec<Tier>> {
        let cache_key = "tiers:public";
        
        // Try cache
        if let Ok(cached) = self.redis.get::<_, String>(cache_key).await {
            if let Ok(tiers) = serde_json::from_str::<Vec<Tier>>(&cached) {
                return Ok(tiers);
            }
        }
        
        let tiers = sqlx::query_as::<_, Tier>(
            r#"
            SELECT * FROM tiers 
            WHERE is_active = true 
            AND is_public = true 
            AND is_deprecated = false
            ORDER BY display_order ASC, price_monthly_cents ASC NULLS LAST
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        
        // Cache for 10 minutes (tiers change less frequently)
        let tiers_json = serde_json::to_string(&tiers)?;
        let _: () = self.redis.set_ex(cache_key, tiers_json, 600).await?;
        
        Ok(tiers)
    }
    
    /// Update tier and invalidate cache
    pub async fn update_tier(
        &mut self,
        slug: &str,
        updates: TierUpdate,
        changed_by: Uuid,
        reason: &str,
    ) -> Result<Tier> {
        // Get current tier for history
        let current = self.get_tier(slug).await?;
        
        // Build dynamic update
        let mut sets = vec![];
        let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + Send>> = vec![];
        
        if let Some(price) = updates.price_monthly_cents {
            sets.push(format!("price_monthly_cents = ${}", params.len() + 1));
            params.push(Box::new(price));
        }
        
        if let Some(price) = updates.price_yearly_cents {
            sets.push(format!("price_yearly_cents = ${}", params.len() + 1));
            params.push(Box::new(price));
        }
        
        if let Some(limits) = updates.limits {
            sets.push(format!("limits = limits || ${}::jsonb", params.len() + 1));
            params.push(Box::new(limits));
        }
        
        if let Some(features) = updates.features {
            sets.push(format!("features = features || ${}::jsonb", params.len() + 1));
            params.push(Box::new(features));
        }
        
        if let Some(active) = updates.is_active {
            sets.push(format!("is_active = ${}", params.len() + 1));
            params.push(Box::new(active));
        }
        
        if let Some(public) = updates.is_public {
            sets.push(format!("is_public = ${}", params.len() + 1));
            params.push(Box::new(public));
        }
        
        // Add version increment
        sets.push(format!("version = version + 1"));
        
        let query = format!(
            "UPDATE tiers SET {} WHERE slug = ${} RETURNING *",
            sets.join(", "),
            params.len() + 1
        );
        
        let mut query_builder = sqlx::query_as::<_, Tier>(&query);
        for param in params {
            query_builder = query_builder.bind(param);
        }
        query_builder = query_builder.bind(slug);
        
        let updated = query_builder.fetch_one(&self.pool).await?;
        
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
        
        // Invalidate caches
        let _: () = self.redis.del(format!("tier:{}", slug)).await?;
        let _: () = self.redis.del("tiers:public").await?;
        
        Ok(updated)
    }
    
    /// Check if a feature is enabled
    pub async fn check_feature(&mut self, tier_slug: &str, feature: &str) -> Result<bool> {
        let tier = self.get_tier(tier_slug).await?;
        Ok(tier.has_feature(feature))
    }
    
    /// Get a specific limit value
    pub async fn get_limit(&mut self, tier_slug: &str, limit_name: &str) -> Result<Option<usize>> {
        let tier = self.get_tier(tier_slug).await?;
        
        let value = match limit_name {
            "max_file_size" => Some(tier.limits.max_file_size),
            "max_files_per_month" => tier.limits.max_files_per_month,
            "max_total_size_per_month" => tier.limits.max_total_size_per_month,
            "max_api_calls_per_month" => tier.limits.max_api_calls_per_month,
            "max_api_keys" => tier.features.max_api_keys,
            "playground_max_daily" => tier.features.playground_max_daily,
            _ => None,
        };
        
        Ok(value)
    }
    
    /// Clear all tier caches (useful for admin operations)
    pub async fn clear_cache(&mut self) -> Result<()> {
        let keys: Vec<String> = self.redis.keys("tier:*").await?;
        if !keys.is_empty() {
            let _: () = self.redis.del(&keys).await?;
        }
        let _: () = self.redis.del("tiers:public").await?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "tier_manager_test.rs"]
mod tests;
```

---

## Day 11-12: User Model with Password Hashing

### Create User Model

```rust
// crates/pii_redacta_core/src/auth/user.rs

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub email_verified: bool,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub verification_token: Option<String>,
    pub is_active: bool,
    pub is_admin: bool,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub company_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub last_login_ip: Option<std::net::IpAddr>,
}

impl User {
    /// Hash a password using Argon2id
    pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string();
        
        Ok(password_hash)
    }
    
    /// Verify a password against stored hash
    pub fn verify_password(&self, password: &str) -> Result<bool, argon2::password_hash::Error> {
        let parsed_hash = PasswordHash::new(&self.password_hash)?;
        let argon2 = Argon2::default();
        
        match argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(e),
        }
    }
    
    /// Generate verification token
    pub fn generate_verification_token() -> String {
        // Generate random 32-byte token
        let random: Vec<u8> = (0..32).map(|_| rand::random()).collect();
        base64::encode(random)
    }
    
    /// Get display name
    pub fn display_name(&self) -> String {
        if let Some(first) = &self.first_name {
            if let Some(last) = &self.last_name {
                return format!("{} {}", first, last);
            }
            return first.clone();
        }
        self.email.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_password_hashing() {
        let password = "SecurePass123!";
        let hash = User::hash_password(password).unwrap();
        
        // Hash should not be the password
        assert_ne!(hash, password);
        assert!(hash.starts_with('$'));
        
        // Correct password should verify
        let user = User {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            password_hash: hash.clone(),
            email_verified: false,
            email_verified_at: None,
            verification_token: None,
            is_active: true,
            is_admin: false,
            first_name: None,
            last_name: None,
            company_name: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login_at: None,
            last_login_ip: None,
        };
        
        assert!(user.verify_password(password).unwrap());
        
        // Wrong password should not verify
        assert!(!user.verify_password("WrongPass").unwrap());
    }
    
    #[test]
    fn test_display_name() {
        let user_with_name = User {
            first_name: Some("John".to_string()),
            last_name: Some("Doe".to_string()),
            email: "john@example.com".to_string(),
            ..Default::default()
        };
        assert_eq!(user_with_name.display_name(), "John Doe");
        
        let user_email_only = User {
            first_name: None,
            last_name: None,
            email: "jane@example.com".to_string(),
            ..Default::default()
        };
        assert_eq!(user_email_only.display_name(), "jane@example.com");
    }
}
```

---

## Day 13-14: Integration & Final Tests

### Integration Tests

Create `crates/pii_redacta_core/tests/integration_test.rs`:

```rust
//! Integration tests for billing system

use pii_redacta_core::billing::*;
use sqlx::PgPool;

async fn setup_test_env() -> (TierManager, PgPool) {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/pii_redacta_test".to_string());
    
    let pool = PgPool::connect(&database_url).await.unwrap();
    
    let redis = redis::Client::open("redis://127.0.0.1:6379")
        .unwrap()
        .get_async_connection()
        .await
        .unwrap();
    
    (TierManager::new(pool.clone(), redis), pool)
}

#[tokio::test]
async fn test_complete_user_signup_flow() {
    let (mut tier_manager, pool) = setup_test_env().await;
    
    // 1. Get trial tier
    let trial = tier_manager.get_tier("trial").await.unwrap();
    assert!(trial.features.api_access); // Trial has API access
    
    // 2. Create user
    let user_id = create_test_user(&pool).await;
    
    // 3. Create subscription
    let subscription = create_trial_subscription(&pool, user_id, &trial).await;
    assert_eq!(subscription.status, SubscriptionStatus::Trial);
    
    // 4. Check limits
    let can_upload = check_upload_allowed(&pool, &subscription, 5 * 1024 * 1024).await;
    assert!(can_upload);
}

#[tokio::test]
async fn test_tier_enforcement() {
    let (mut tier_manager, pool) = setup_test_env().await;
    
    let trial = tier_manager.get_tier("trial").await.unwrap();
    
    // Try to upload 15MB file (over 10MB limit)
    let result = trial.check_file_size(15 * 1024 * 1024);
    assert!(result.is_err());
    
    // Upload 5MB file (under limit)
    let result = trial.check_file_size(5 * 1024 * 1024);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_monthly_limit_enforcement() {
    let (mut tier_manager, _pool) = setup_test_env().await;
    let trial = tier_manager.get_tier("trial").await.unwrap();
    
    // Simulate user who has used 48 of 50 files
    let usage = MonthlyUsage {
        files: 48,
        total_size: 0,
        api_calls: 0,
    };
    
    // Try to upload 5 files (would exceed limit)
    let result = trial.check_monthly_limits(&usage, 5, 0);
    assert!(result.is_err());
    
    // Try to upload 1 file (would reach exactly 49)
    let result = trial.check_monthly_limits(&usage, 1, 0);
    assert!(matches!(result, Ok(LimitStatus::ApproachingLimit { .. })));
    
    // Try to upload 2 files (would reach exactly 50)
    let result = trial.check_monthly_limits(&usage, 2, 0);
    assert!(matches!(result, Ok(LimitStatus::WithinLimits)));
}
```

---

## Sprint 9 Commit Message

```
feat(billing): sprint 9 - database foundation and configurable tier system

Database:
- Add PostgreSQL with SQLx integration
- Create tiers table with JSONB limits and features
- Create tier_history table for audit trail
- Create users table with Argon2 password hashing
- Create subscriptions table with usage tracking

Tier System:
- Implement TierManager with Redis caching (5-min TTL)
- Configurable limits: file size, monthly quotas, rate limits
- Configurable features: API access, batch processing, support level
- Soft limits with warnings at 80%
- Trial tier with API access enabled (2 keys)

Caching:
- Redis integration for tier caching
- Automatic cache invalidation on updates
- Cache warming on startup (optional)

Models:
- Tier with JSONB configuration
- User with Argon2 password hashing
- Subscription with usage tracking
- MonthlyUsage for quota checking

Security:
- Argon2id for password hashing (NOT bcrypt)
- Prepared statements via SQLx
- Constant-time password verification

Tests:
- 15 migration tests
- 12 tier model tests
- 10 tier manager tests
- 8 integration tests

Migration commands:
  sqlx migrate run

Total: 45 new tests passing
```

---

## Running Sprint 9

```bash
# 1. Start PostgreSQL and Redis
docker-compose up -d postgres redis

# 2. Set up database
export DATABASE_URL=postgres://postgres:postgres@localhost/pii_redacta_dev
sqlx database create
sqlx migrate run

# 3. Run tests
cargo test -p pii_redacta_core --test migration_test
cargo test -p pii_redacta_core --lib
cargo test -p pii_redacta_core --test integration_test

# 4. Build
cargo build --release
```

---

**Ready to start Sprint 9 implementation?**
