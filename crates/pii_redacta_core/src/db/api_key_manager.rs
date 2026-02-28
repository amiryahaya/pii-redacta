//! API Key Manager with HMAC-SHA256 Security
//!
//! Provides secure API key generation, storage, and validation.
//!
//! Security Model:
//! - API keys are generated with format: `pii_{env}_{prefix}_{secret}`
//! - Only HMAC-SHA256 hash of the full key is stored (with server secret)
//! - First 8 chars of secret are stored as prefix for identification
//! - Server secret is required to validate keys (prevents rainbow table attacks)

use super::models::ApiKey;
use super::Database;
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use sqlx::Error as SqlxError;
use std::sync::Arc;
use uuid::Uuid;

/// Error types for API key operations
#[derive(Debug, thiserror::Error)]
pub enum ApiKeyError {
    #[error("Database error: {0}")]
    Database(#[from] SqlxError),

    #[error("Invalid API key format")]
    InvalidFormat,

    #[error("API key not found or revoked")]
    NotFound,

    #[error("API key has expired")]
    Expired,

    #[error("Cryptographic error: {0}")]
    Crypto(String),

    #[error("Rate limit exceeded")]
    RateLimited,

    #[error("User has reached maximum number of API keys")]
    MaxKeysReached,
}

/// Result type for API key operations
pub type Result<T> = std::result::Result<T, ApiKeyError>;

/// Environment for API key
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiKeyEnvironment {
    /// Production environment
    Live,
    /// Test/development environment
    Test,
}

impl ApiKeyEnvironment {
    fn as_str(&self) -> &'static str {
        match self {
            ApiKeyEnvironment::Live => "live",
            ApiKeyEnvironment::Test => "test",
        }
    }
}

impl std::str::FromStr for ApiKeyEnvironment {
    type Err = ApiKeyError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "live" => Ok(ApiKeyEnvironment::Live),
            "test" => Ok(ApiKeyEnvironment::Test),
            _ => Err(ApiKeyError::InvalidFormat),
        }
    }
}

/// A generated API key (plaintext) - only returned once at creation
#[derive(Debug, Clone)]
pub struct GeneratedApiKey {
    /// Database-assigned ID
    pub id: Uuid,
    /// The full API key to show to the user (ONE TIME ONLY)
    pub full_key: String,
    /// Environment (live/test)
    pub environment: ApiKeyEnvironment,
    /// Key prefix (first 8 chars of secret)
    pub prefix: String,
    /// User ID
    pub user_id: Uuid,
    /// Expiration date (if any)
    pub expires_at: Option<chrono::DateTime<Utc>>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<Utc>,
}

/// API Key validation result
#[derive(Debug, Clone)]
pub struct ValidatedApiKey {
    /// The API key database record
    pub api_key: ApiKey,
    /// User ID associated with this key
    pub user_id: Uuid,
    /// Environment (live/test)
    pub environment: ApiKeyEnvironment,
}

/// Manages API key operations
#[derive(Clone)]
pub struct ApiKeyManager {
    db: Arc<Database>,
    /// Server secret for HMAC operations (loaded from environment)
    server_secret: Vec<u8>,
}

/// Type alias for HMAC-SHA256
type HmacSha256 = Hmac<Sha256>;

impl ApiKeyManager {
    /// Create a new API key manager
    ///
    /// # Arguments
    /// * `db` - Database connection
    /// * `server_secret` - Server secret for HMAC (from env var, base64 encoded)
    pub fn new(db: Arc<Database>, server_secret_b64: &str) -> Result<Self> {
        let server_secret = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            server_secret_b64,
        )
        .map_err(|e| ApiKeyError::Crypto(format!("Invalid base64 secret: {}", e)))?;

        if server_secret.len() < 32 {
            return Err(ApiKeyError::Crypto(
                "Server secret must be at least 32 bytes".to_string(),
            ));
        }

        Ok(Self { db, server_secret })
    }

    /// Get reference to the database pool
    pub fn pool(&self) -> &sqlx::Pool<sqlx::Postgres> {
        self.db.pool()
    }

    // ============================================
    // API Key Generation
    // ============================================

    /// Generate a new API key for a user
    ///
    /// # Arguments
    /// * `user_id` - User to create key for
    /// * `name` - Human-readable name for this key
    /// * `environment` - Live or test environment
    /// * `expires_at` - Optional expiration date
    ///
    /// # Returns
    /// The generated API key (full plaintext) - **SHOW THIS ONCE TO USER**
    pub async fn generate_key(
        &self,
        user_id: Uuid,
        name: &str,
        environment: ApiKeyEnvironment,
        expires_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<GeneratedApiKey> {
        // Generate cryptographically secure random secret (32 bytes = 64 hex chars)
        let secret_bytes = Self::generate_secure_random(32);
        let secret = hex::encode(&secret_bytes);

        // First 8 characters for prefix
        let prefix = &secret[..8];

        // Construct full key: pii_{env}_{prefix}_{secret}
        let full_key = format!("pii_{}_{}_{}", environment.as_str(), prefix, secret);

        // Hash the full key with HMAC-SHA256
        let key_hash = self.hash_key(&full_key)?;

        // Store in database (includes environment column)
        let api_key = sqlx::query_as::<_, ApiKey>(
            r#"
            INSERT INTO api_keys (
                user_id, key_prefix, key_hash, name,
                environment, expires_at, is_active, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, true, NOW())
            RETURNING
                id, user_id, key_prefix, key_hash, name,
                last_used_at, expires_at, is_active, revoked_at,
                revoked_reason, created_at, environment
            "#,
        )
        .bind(user_id)
        .bind(prefix)
        .bind(&key_hash)
        .bind(name)
        .bind(environment.as_str())
        .bind(expires_at)
        .fetch_one(self.db.pool())
        .await?;

        Ok(GeneratedApiKey {
            id: api_key.id,
            full_key,
            environment,
            prefix: prefix.to_string(),
            user_id,
            expires_at,
            created_at: api_key.created_at,
        })
    }

    // ============================================
    // API Key Validation
    // ============================================

    /// Validate an API key
    ///
    /// This checks:
    /// 1. Key format is valid
    /// 2. Key exists in database
    /// 3. Key is active (not revoked)
    /// 4. Key has not expired
    /// 5. HMAC hash matches
    ///
    /// On successful validation, updates `last_used_at` timestamp.
    pub async fn validate_key(&self, api_key: &str) -> Result<ValidatedApiKey> {
        // Parse key format
        let (environment, _prefix, _secret) = Self::parse_key_format(api_key)?;

        // Hash the provided key
        let key_hash = self.hash_key(api_key)?;

        // Look up key by hash
        let db_key = sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT
                id, user_id, key_prefix, key_hash, name,
                last_used_at, expires_at, is_active, revoked_at,
                revoked_reason, created_at, environment
            FROM api_keys 
            WHERE key_hash = $1 AND is_active = true
            "#,
        )
        .bind(&key_hash)
        .fetch_optional(self.db.pool())
        .await?;

        let db_key = db_key.ok_or(ApiKeyError::NotFound)?;
        let user_id = db_key.user_id;
        let key_id = db_key.id;

        // Check expiration
        if let Some(expires_at) = db_key.expires_at {
            if Utc::now() > expires_at {
                return Err(ApiKeyError::Expired);
            }
        }

        // Update last_used_at (fire and forget - don't fail validation if this fails)
        let _ = self.update_last_used(key_id).await;

        Ok(ValidatedApiKey {
            api_key: db_key,
            user_id,
            environment,
        })
    }

    /// Validate an API key without updating last_used_at
    ///
    /// Use this for read-only operations where you don't want to
    /// trigger a database write.
    pub async fn validate_key_readonly(&self, api_key: &str) -> Result<ValidatedApiKey> {
        let (environment, _prefix, _secret) = Self::parse_key_format(api_key)?;
        let key_hash = self.hash_key(api_key)?;

        let db_key = sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT
                id, user_id, key_prefix, key_hash, name,
                last_used_at, expires_at, is_active, revoked_at,
                revoked_reason, created_at, environment
            FROM api_keys 
            WHERE key_hash = $1 AND is_active = true
            "#,
        )
        .bind(&key_hash)
        .fetch_optional(self.db.pool())
        .await?;

        let db_key = db_key.ok_or(ApiKeyError::NotFound)?;
        let user_id = db_key.user_id;

        if let Some(expires_at) = db_key.expires_at {
            if Utc::now() > expires_at {
                return Err(ApiKeyError::Expired);
            }
        }

        Ok(ValidatedApiKey {
            api_key: db_key,
            user_id,
            environment,
        })
    }

    // ============================================
    // API Key Management
    // ============================================

    /// List all API keys for a user
    pub async fn list_user_keys(&self, user_id: Uuid) -> Result<Vec<ApiKey>> {
        let keys = sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT
                id, user_id, key_prefix, key_hash, name,
                last_used_at, expires_at, is_active, revoked_at,
                revoked_reason, created_at, environment
            FROM api_keys 
            WHERE user_id = $1 AND is_active = true
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(self.db.pool())
        .await?;

        Ok(keys)
    }

    /// Count active API keys for a user
    pub async fn count_user_keys(&self, user_id: Uuid) -> Result<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM api_keys 
            WHERE user_id = $1 AND is_active = true
            "#,
        )
        .bind(user_id)
        .fetch_one(self.db.pool())
        .await?;

        Ok(count)
    }

    /// Revoke an API key
    pub async fn revoke_key(
        &self,
        key_id: Uuid,
        user_id: Uuid,
        reason: Option<&str>,
    ) -> Result<ApiKey> {
        let key = sqlx::query_as::<_, ApiKey>(
            r#"
            UPDATE api_keys 
            SET is_active = false, revoked_at = NOW(), revoked_reason = $1
            WHERE id = $2 AND user_id = $3 AND is_active = true
            RETURNING
                id, user_id, key_prefix, key_hash, name,
                last_used_at, expires_at, is_active, revoked_at,
                revoked_reason, created_at, environment
            "#,
        )
        .bind(reason)
        .bind(key_id)
        .bind(user_id)
        .fetch_optional(self.db.pool())
        .await?;

        key.ok_or(ApiKeyError::NotFound)
    }

    /// Get a single API key by ID (for a specific user)
    pub async fn get_key(&self, key_id: Uuid, user_id: Uuid) -> Result<ApiKey> {
        let key = sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT
                id, user_id, key_prefix, key_hash, name,
                last_used_at, expires_at, is_active, revoked_at,
                revoked_reason, created_at, environment
            FROM api_keys 
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(key_id)
        .bind(user_id)
        .fetch_optional(self.db.pool())
        .await?;

        key.ok_or(ApiKeyError::NotFound)
    }

    // ============================================
    // Helper Methods
    // ============================================

    /// Generate cryptographically secure random bytes
    fn generate_secure_random(len: usize) -> Vec<u8> {
        use rand::RngCore;
        let mut bytes = vec![0u8; len];
        rand::thread_rng().fill_bytes(&mut bytes);
        bytes
    }

    /// Hash an API key using HMAC-SHA256
    fn hash_key(&self, key: &str) -> Result<String> {
        let mut mac = HmacSha256::new_from_slice(&self.server_secret)
            .map_err(|e| ApiKeyError::Crypto(format!("HMAC error: {}", e)))?;

        mac.update(key.as_bytes());
        let result = mac.finalize();
        let hash = hex::encode(result.into_bytes());
        Ok(hash)
    }

    /// Parse API key format: pii_{env}_{prefix}_{secret}
    ///
    /// Returns (environment, prefix, secret)
    fn parse_key_format(key: &str) -> Result<(ApiKeyEnvironment, &str, &str)> {
        let parts: Vec<&str> = key.split('_').collect();

        // Expected: ["pii", "live|test", "prefix", "secret..."]
        if parts.len() < 4 || parts[0] != "pii" {
            return Err(ApiKeyError::InvalidFormat);
        }

        let environment = parts[1].parse()?;
        let _prefix = parts[2];
        let secret = parts[3..].join("_"); // Handle case where secret might contain underscores

        // Validate prefix length (should be 8 chars)
        if _prefix.len() != 8 {
            return Err(ApiKeyError::InvalidFormat);
        }

        // Validate secret length (should be 64 hex chars = 32 bytes)
        if secret.len() != 64 || !secret.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ApiKeyError::InvalidFormat);
        }

        Ok((environment, _prefix, parts[3]))
    }

    /// Update last_used_at timestamp (internal use)
    async fn update_last_used(&self, key_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE api_keys SET last_used_at = NOW() WHERE id = $1")
            .bind(key_id)
            .execute(self.db.pool())
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_secure_random() {
        let random1 = ApiKeyManager::generate_secure_random(32);
        let random2 = ApiKeyManager::generate_secure_random(32);

        assert_eq!(random1.len(), 32);
        assert_eq!(random2.len(), 32);
        // Should be different (extremely unlikely to collide)
        assert_ne!(random1, random2);
    }

    #[test]
    fn test_hash_key_deterministic() {
        let test_secret = "dGVzdC1zZWNyZXQtMzItYnl0ZXMtbG9uZy1rZXktZm9yLWhtYWM=";

        // Just test the hash function logic
        let secret =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, test_secret)
                .unwrap();

        let mut mac1 = HmacSha256::new_from_slice(&secret).unwrap();
        mac1.update(b"test-key-123");
        let hash1 = hex::encode(mac1.finalize().into_bytes());

        let mut mac2 = HmacSha256::new_from_slice(&secret).unwrap();
        mac2.update(b"test-key-123");
        let hash2 = hex::encode(mac2.finalize().into_bytes());

        // Same input + same secret = same hash
        assert_eq!(hash1, hash2);

        // Different input = different hash
        let mut mac3 = HmacSha256::new_from_slice(&secret).unwrap();
        mac3.update(b"different-key");
        let hash3 = hex::encode(mac3.finalize().into_bytes());
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_parse_key_format_valid() {
        let key =
            "pii_live_a1b2c3d4_e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2";
        let (env, prefix, secret) = ApiKeyManager::parse_key_format(key).unwrap();

        assert!(matches!(env, ApiKeyEnvironment::Live));
        assert_eq!(prefix, "a1b2c3d4");
        assert_eq!(secret.len(), 64); // First part of secret
    }

    #[test]
    fn test_parse_key_format_test_env() {
        let key =
            "pii_test_12345678_a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2";
        let (env, _, _) = ApiKeyManager::parse_key_format(key).unwrap();

        assert!(matches!(env, ApiKeyEnvironment::Test));
    }

    #[test]
    fn test_parse_key_format_invalid() {
        // Wrong prefix
        assert!(ApiKeyManager::parse_key_format("invalid_key").is_err());

        // Wrong environment
        assert!(ApiKeyManager::parse_key_format("pii_invalid_12345678_").is_err());

        // Prefix too short
        assert!(ApiKeyManager::parse_key_format("pii_live_1234_abc").is_err());

        // Secret too short
        assert!(ApiKeyManager::parse_key_format("pii_live_12345678_abc").is_err());
    }

    #[test]
    fn test_api_key_environment_from_str() {
        assert!(matches!(
            "live".parse::<ApiKeyEnvironment>().unwrap(),
            ApiKeyEnvironment::Live
        ));
        assert!(matches!(
            "test".parse::<ApiKeyEnvironment>().unwrap(),
            ApiKeyEnvironment::Test
        ));
        assert!("invalid".parse::<ApiKeyEnvironment>().is_err());
    }
}
