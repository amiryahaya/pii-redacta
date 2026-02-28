//! Database models for PII Redacta
//!
//! This module contains structs that map directly to database tables.
//! All timestamp fields use chrono::DateTime<Utc> for proper timezone handling.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ============================================
// Tier Models
// ============================================

/// Represents a pricing tier/plan in the database
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Tier {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,

    /// JSONB limits configuration
    pub limits: sqlx::types::Json<TierLimits>,

    /// JSONB features configuration
    pub features: sqlx::types::Json<TierFeatures>,

    pub monthly_price_cents: Option<i32>,
    pub yearly_price_cents: Option<i32>,

    pub is_public: bool,
    pub is_active: bool,
    pub sort_order: i32,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Tier limits configuration (stored as JSONB)
///
/// All limits are optional (None = unlimited for enterprise tiers)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct TierLimits {
    /// API access enabled
    #[serde(default)]
    pub api_enabled: bool,

    /// Maximum number of API keys (None = unlimited)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_api_keys: Option<i32>,

    /// Maximum file size in bytes (None = unlimited)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_file_size: Option<i64>,

    /// Maximum files per month (None = unlimited)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_files_per_month: Option<i32>,

    /// Maximum pages per file (None = unlimited)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_pages_per_file: Option<i32>,

    /// Maximum total storage in bytes (None = unlimited)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_total_size: Option<i64>,

    /// Maximum playground uses per day (None = unlimited)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playground_max_daily: Option<i32>,

    /// Maximum file size for playground in bytes (None = unlimited)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playground_max_file_size: Option<i64>,

    /// Data retention period in days (None = unlimited)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retention_days: Option<i32>,
}

/// Tier features configuration (stored as JSONB)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct TierFeatures {
    /// Enable batch processing
    #[serde(default)]
    pub batch_processing: bool,

    /// Allow custom detection rules
    #[serde(default)]
    pub custom_rules: bool,

    /// Email support available
    #[serde(default)]
    pub email_support: bool,

    /// Playground access enabled
    #[serde(default = "default_true")]
    pub playground: bool,

    /// Rate limit per minute (None = unlimited)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit_per_minute: Option<i32>,

    /// SLA guarantee (e.g., "99%", "99.9%", None = no SLA)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sla: Option<String>,

    /// Webhook support
    #[serde(default)]
    pub webhooks: bool,
}

fn default_true() -> bool {
    true
}

// ============================================
// User Models
// ============================================

/// User account in the database
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub email_verified_at: Option<DateTime<Utc>>,

    /// Argon2id password hash
    pub password_hash: Option<String>,

    pub display_name: Option<String>,
    pub company_name: Option<String>,

    pub email_notifications_enabled: bool,
    pub is_admin: bool,
    pub last_login_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// ============================================
// Subscription Models
// ============================================

/// Subscription status
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Trial,
    Active,
    PastDue,
    Cancelled,
    Expired,
}

/// User subscription linking to a tier
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Subscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tier_id: Uuid,

    pub status: SubscriptionStatus,

    pub current_period_start: Option<DateTime<Utc>>,
    pub current_period_end: Option<DateTime<Utc>>,

    pub cancel_at_period_end: bool,
    pub cancelled_at: Option<DateTime<Utc>>,

    /// Stripe customer ID (optional)
    pub stripe_customer_id: Option<String>,
    /// Stripe subscription ID (optional)
    pub stripe_subscription_id: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================
// API Key Models
// ============================================

/// API key for programmatic access
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,

    /// First 8 characters of the key for display
    pub key_prefix: String,

    /// HMAC-SHA256 hash of the full key
    pub key_hash: String,

    pub name: String,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,

    pub is_active: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_reason: Option<String>,

    pub created_at: DateTime<Utc>,

    /// Environment: "live" or "test"
    #[sqlx(default)]
    #[serde(default = "default_environment")]
    pub environment: String,
}

fn default_environment() -> String {
    "live".to_string()
}

// ============================================
// Usage Log Models
// ============================================

/// Type of API request
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum RequestType {
    Playground,
    PlaygroundFile,
    ApiDetect,
    ApiRedact,
    ApiDetectStream,
    FileUpload,
}

/// Usage log entry for analytics and limits
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UsageLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub api_key_id: Option<Uuid>,

    pub request_type: RequestType,

    pub file_name: Option<String>,
    pub file_size_bytes: Option<i32>,
    pub file_type: Option<String>,

    pub processing_time_ms: Option<i32>,
    pub page_count: Option<i32>,
    pub detections_count: Option<i32>,

    pub success: bool,
    pub error_message: Option<String>,

    /// Client IP address for rate limiting
    pub ip_address: Option<std::net::IpAddr>,

    pub created_at: DateTime<Utc>,
}

// ============================================
// IP Block Models
// ============================================

/// IP address block for security
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct IpBlock {
    pub id: Uuid,

    /// Single IP address
    pub ip_address: Option<std::net::IpAddr>,
    /// IP range (CIDR notation)
    pub ip_range: Option<ipnetwork::IpNetwork>,

    pub reason: String,
    pub blocked_by: Option<Uuid>,

    pub expires_at: Option<DateTime<Utc>>,

    pub is_active: bool,
    pub hit_count: i32,
    pub last_hit_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
}

// ============================================
// Helper Methods
// ============================================

impl TierLimits {
    /// Check if a file size is within limits
    pub fn is_file_size_allowed(&self, size_bytes: i64) -> bool {
        match self.max_file_size {
            None => true,
            Some(max) => size_bytes <= max,
        }
    }

    /// Check if total storage would be exceeded
    pub fn is_storage_available(&self, current_usage: i64, additional: i64) -> bool {
        match self.max_total_size {
            None => true,
            Some(max) => current_usage + additional <= max,
        }
    }

    /// Get human-readable file size limit
    pub fn max_file_size_human(&self) -> String {
        match self.max_file_size {
            None => "Unlimited".to_string(),
            Some(bytes) => format_size(bytes),
        }
    }
}

impl TierFeatures {
    /// Check if a feature is available
    pub fn has_feature(&self, feature: &str) -> bool {
        match feature {
            "batch_processing" => self.batch_processing,
            "custom_rules" => self.custom_rules,
            "email_support" => self.email_support,
            "playground" => self.playground,
            "webhooks" => self.webhooks,
            _ => false,
        }
    }
}

/// Format bytes to human-readable string
fn format_size(bytes: i64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_idx])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_limits_file_size() {
        let limits = TierLimits {
            max_file_size: Some(10_485_760), // 10MB
            ..Default::default()
        };

        assert!(limits.is_file_size_allowed(5_000_000));
        assert!(limits.is_file_size_allowed(10_485_760));
        assert!(!limits.is_file_size_allowed(20_000_000));
    }

    #[test]
    fn test_tier_limits_unlimited() {
        let limits = TierLimits {
            max_file_size: None,
            max_files_per_month: None,
            ..Default::default()
        };

        assert!(limits.is_file_size_allowed(1_000_000_000_000)); // 1TB
    }

    #[test]
    fn test_tier_storage_available() {
        let limits = TierLimits {
            max_total_size: Some(500_000_000), // 500MB
            ..Default::default()
        };

        assert!(limits.is_storage_available(100_000_000, 100_000_000));
        assert!(!limits.is_storage_available(400_000_000, 200_000_000));
    }

    #[test]
    fn test_tier_features_check() {
        let features = TierFeatures {
            batch_processing: true,
            webhooks: true,
            ..Default::default()
        };

        assert!(features.has_feature("batch_processing"));
        assert!(features.has_feature("webhooks"));
        assert!(!features.has_feature("custom_rules"));
        assert!(!features.has_feature("unknown"));
    }
}
