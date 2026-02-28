//! Tier Manager with Redis caching
//!
//! Provides efficient tier lookups with automatic cache invalidation.
//! Cache TTL: 5 minutes for tier configurations.

use super::models::{Tier, TierFeatures, TierLimits};
use super::Database;
use redis::{AsyncCommands, Client as RedisClient};
use sqlx::Error as SqlxError;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// Default cache TTL for tier data (5 minutes)
const TIER_CACHE_TTL: u64 = 300;
/// Redis key prefix for tier data
const TIER_CACHE_PREFIX: &str = "tier:";
/// Redis key for tier list
const TIER_LIST_CACHE_KEY: &str = "tiers:active";

/// Error types for TierManager operations
#[derive(Debug, thiserror::Error)]
pub enum TierManagerError {
    #[error("Database error: {0}")]
    Database(#[from] SqlxError),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Tier not found: {0}")]
    NotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type for TierManager operations
pub type Result<T> = std::result::Result<T, TierManagerError>;

/// Manages tier data with caching support
#[derive(Clone)]
pub struct TierManager {
    db: Arc<Database>,
    redis: Option<RedisClient>,
    cache_ttl: Duration,
}

impl TierManager {
    /// Create a new TierManager without caching
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            redis: None,
            cache_ttl: Duration::from_secs(TIER_CACHE_TTL),
        }
    }

    /// Create a new TierManager with Redis caching
    pub fn with_redis(db: Arc<Database>, redis_url: &str) -> Result<Self> {
        let redis = RedisClient::open(redis_url)?;
        Ok(Self {
            db,
            redis: Some(redis),
            cache_ttl: Duration::from_secs(TIER_CACHE_TTL),
        })
    }

    /// Set custom cache TTL (default: 5 minutes)
    pub fn with_cache_ttl(mut self, ttl: Duration) -> Self {
        self.cache_ttl = ttl;
        self
    }

    // ============================================
    // Tier Retrieval
    // ============================================

    /// Get a tier by its ID
    ///
    /// First checks Redis cache, then falls back to database.
    pub async fn get_by_id(&self, tier_id: Uuid) -> Result<Tier> {
        // Try cache first
        if let Some(redis) = &self.redis {
            let cache_key = format!("{}{}", TIER_CACHE_PREFIX, tier_id);
            let mut conn = redis.get_multiplexed_async_connection().await?;

            if let Some(cached) = conn.get::<_, Option<String>>(&cache_key).await? {
                let tier: Tier = serde_json::from_str(&cached)?;
                return Ok(tier);
            }
        }

        // Fetch from database
        let tier = sqlx::query_as::<_, Tier>(
            r#"
            SELECT 
                id, name, display_name, description,
                limits,
                features,
                monthly_price_cents, yearly_price_cents,
                is_public, is_active, sort_order,
                created_at, updated_at
            FROM tiers 
            WHERE id = $1 AND is_active = true
            "#,
        )
        .bind(tier_id)
        .fetch_optional(self.db.pool())
        .await?;

        match tier {
            Some(tier) => {
                // Cache the result
                self.cache_tier(&tier).await?;
                Ok(tier)
            }
            None => Err(TierManagerError::NotFound(tier_id.to_string())),
        }
    }

    /// Get a tier by its name (e.g., "trial", "starter")
    ///
    /// First checks Redis cache, then falls back to database.
    pub async fn get_by_name(&self, name: &str) -> Result<Tier> {
        // Try cache first (using name-based key)
        if let Some(redis) = &self.redis {
            let cache_key = format!("{}name:{}", TIER_CACHE_PREFIX, name);
            let mut conn = redis.get_multiplexed_async_connection().await?;

            if let Some(cached) = conn.get::<_, Option<String>>(&cache_key).await? {
                let tier: Tier = serde_json::from_str(&cached)?;
                return Ok(tier);
            }
        }

        // Fetch from database
        let tier = sqlx::query_as::<_, Tier>(
            r#"
            SELECT 
                id, name, display_name, description,
                limits,
                features,
                monthly_price_cents, yearly_price_cents,
                is_public, is_active, sort_order,
                created_at, updated_at
            FROM tiers 
            WHERE name = $1 AND is_active = true
            "#,
        )
        .bind(name)
        .fetch_optional(self.db.pool())
        .await?;

        match tier {
            Some(tier) => {
                // Cache the result (both by ID and name)
                self.cache_tier(&tier).await?;
                Ok(tier)
            }
            None => Err(TierManagerError::NotFound(name.to_string())),
        }
    }

    /// Get all active, public tiers ordered by sort_order
    pub async fn list_active_tiers(&self) -> Result<Vec<Tier>> {
        // Try cache first
        if let Some(redis) = &self.redis {
            let mut conn = redis.get_multiplexed_async_connection().await?;

            if let Some(cached) = conn.get::<_, Option<String>>(TIER_LIST_CACHE_KEY).await? {
                let tiers: Vec<Tier> = serde_json::from_str(&cached)?;
                return Ok(tiers);
            }
        }

        // Fetch from database
        let tiers = sqlx::query_as::<_, Tier>(
            r#"
            SELECT 
                id, name, display_name, description,
                limits,
                features,
                monthly_price_cents, yearly_price_cents,
                is_public, is_active, sort_order,
                created_at, updated_at
            FROM tiers 
            WHERE is_active = true AND is_public = true
            ORDER BY sort_order ASC
            "#,
        )
        .fetch_all(self.db.pool())
        .await?;

        // Cache the list
        if let Some(redis) = &self.redis {
            let cache_value = serde_json::to_string(&tiers)?;
            let mut conn = redis.get_multiplexed_async_connection().await?;
            redis::cmd("SETEX")
                .arg(TIER_LIST_CACHE_KEY)
                .arg(self.cache_ttl.as_secs() as i64)
                .arg(cache_value)
                .query_async::<_, ()>(&mut conn)
                .await?;
        }

        Ok(tiers)
    }

    // ============================================
    // Tier Cache Management
    // ============================================

    /// Cache a tier in Redis (by ID and name)
    async fn cache_tier(&self, tier: &Tier) -> Result<()> {
        if let Some(redis) = &self.redis {
            let cache_value = serde_json::to_string(tier)?;
            let mut conn = redis.get_multiplexed_async_connection().await?;

            // Cache by ID
            let id_key = format!("{}{}", TIER_CACHE_PREFIX, tier.id);
            redis::cmd("SETEX")
                .arg(&id_key)
                .arg(self.cache_ttl.as_secs() as i64)
                .arg(&cache_value)
                .query_async::<_, ()>(&mut conn)
                .await?;

            // Cache by name
            let name_key = format!("{}name:{}", TIER_CACHE_PREFIX, tier.name);
            redis::cmd("SETEX")
                .arg(&name_key)
                .arg(self.cache_ttl.as_secs() as i64)
                .arg(&cache_value)
                .query_async::<_, ()>(&mut conn)
                .await?;
        }

        Ok(())
    }

    /// Invalidate tier cache
    ///
    /// Call this when a tier is updated.
    pub async fn invalidate_cache(&self, tier_id: Uuid, tier_name: Option<&str>) -> Result<()> {
        if let Some(redis) = &self.redis {
            let mut conn = redis.get_multiplexed_async_connection().await?;

            // Delete by ID
            let id_key = format!("{}{}", TIER_CACHE_PREFIX, tier_id);
            conn.del::<_, ()>(&id_key).await?;

            // Delete by name if provided
            if let Some(name) = tier_name {
                let name_key = format!("{}name:{}", TIER_CACHE_PREFIX, name);
                conn.del::<_, ()>(&name_key).await?;
            }

            // Invalidate tier list cache
            conn.del::<_, ()>(TIER_LIST_CACHE_KEY).await?;
        }

        Ok(())
    }

    /// Invalidate all tier caches (S9-R4-05: uses SCAN instead of KEYS to avoid blocking Redis)
    pub async fn invalidate_all_cache(&self) -> Result<()> {
        if let Some(redis) = &self.redis {
            let mut conn = redis.get_multiplexed_async_connection().await?;

            // Use SCAN for non-blocking iteration instead of KEYS (which blocks Redis)
            let pattern = format!("{}*", TIER_CACHE_PREFIX);
            let mut cursor: u64 = 0;
            loop {
                let (next_cursor, batch): (u64, Vec<String>) = redis::cmd("SCAN")
                    .arg(cursor)
                    .arg("MATCH")
                    .arg(&pattern)
                    .arg("COUNT")
                    .arg(100)
                    .query_async(&mut conn)
                    .await?;

                if !batch.is_empty() {
                    conn.del::<_, ()>(&batch).await?;
                }
                if next_cursor == 0 {
                    break;
                }
                cursor = next_cursor;
            }

            // Also delete tier list
            conn.del::<_, ()>(TIER_LIST_CACHE_KEY).await?;
        }

        Ok(())
    }

    // ============================================
    // Tier Updates (Admin only)
    // ============================================

    /// Update a tier's limits
    ///
    /// Automatically invalidates the cache after update.
    pub async fn update_limits(&self, tier_id: Uuid, limits: &TierLimits) -> Result<Tier> {
        let limits_json = serde_json::to_value(limits)?;

        let tier = sqlx::query_as::<_, Tier>(
            r#"
            UPDATE tiers 
            SET limits = $1, updated_at = NOW()
            WHERE id = $2 AND is_active = true
            RETURNING 
                id, name, display_name, description,
                limits,
                features,
                monthly_price_cents, yearly_price_cents,
                is_public, is_active, sort_order,
                created_at, updated_at
            "#,
        )
        .bind(limits_json)
        .bind(tier_id)
        .fetch_optional(self.db.pool())
        .await?;

        match tier {
            Some(tier) => {
                self.invalidate_cache(tier_id, Some(&tier.name)).await?;
                Ok(tier)
            }
            None => Err(TierManagerError::NotFound(tier_id.to_string())),
        }
    }

    /// Update a tier's features
    ///
    /// Automatically invalidates the cache after update.
    pub async fn update_features(&self, tier_id: Uuid, features: &TierFeatures) -> Result<Tier> {
        let features_json = serde_json::to_value(features)?;

        let tier = sqlx::query_as::<_, Tier>(
            r#"
            UPDATE tiers 
            SET features = $1, updated_at = NOW()
            WHERE id = $2 AND is_active = true
            RETURNING 
                id, name, display_name, description,
                limits,
                features,
                monthly_price_cents, yearly_price_cents,
                is_public, is_active, sort_order,
                created_at, updated_at
            "#,
        )
        .bind(features_json)
        .bind(tier_id)
        .fetch_optional(self.db.pool())
        .await?;

        match tier {
            Some(tier) => {
                self.invalidate_cache(tier_id, Some(&tier.name)).await?;
                Ok(tier)
            }
            None => Err(TierManagerError::NotFound(tier_id.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_cache_keys() {
        let tier_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(
            format!("{}{}", TIER_CACHE_PREFIX, tier_id),
            "tier:550e8400-e29b-41d4-a716-446655440000"
        );
        assert_eq!(
            format!("{}name:{}", TIER_CACHE_PREFIX, "trial"),
            "tier:name:trial"
        );
    }

    #[test]
    fn test_default_cache_ttl() {
        assert_eq!(TIER_CACHE_TTL, 300); // 5 minutes
    }
}
