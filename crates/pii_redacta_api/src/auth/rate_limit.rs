//! Rate limiting for API requests
//!
//! Implements sliding window rate limiting using Redis.
//! Supports multiple limit types: per-key, per-user, and per-tier.

use redis::{AsyncCommands, Client as RedisClient};
use std::time::{SystemTime, UNIX_EPOCH};

/// Rate limiter using Redis sliding window
#[derive(Clone)]
pub struct RateLimiter {
    redis: RedisClient,
}

/// Rate limit check result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitResult {
    /// Request is allowed
    Allowed,
    /// Request is denied, retry after this many seconds
    RetryAfter(u64),
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(redis: RedisClient) -> Self {
        Self { redis }
    }

    /// Check if a request is allowed for a given API key
    ///
    /// # Arguments
    /// * `api_key_id` - The API key ID
    /// * `tier_rate_limit` - Max requests per minute from tier config (None = unlimited)
    ///
    /// Returns `RateLimitResult::Allowed` if request should proceed,
    /// or `RateLimitResult::RetryAfter(seconds)` if rate limited.
    pub async fn check_key_limit(
        &self,
        api_key_id: uuid::Uuid,
        tier_rate_limit: Option<i32>,
    ) -> Result<RateLimitResult, redis::RedisError> {
        // Unlimited rate limit
        let limit = match tier_rate_limit {
            Some(l) => l as u64,
            None => return Ok(RateLimitResult::Allowed),
        };

        // Minimum 1 request per minute
        if limit == 0 {
            return Ok(RateLimitResult::RetryAfter(60));
        }

        let mut conn = self.redis.get_multiplexed_async_connection().await?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Window is 1 minute
        let window = 60u64;
        let window_start = now - (now % window);
        let key = format!("rate_limit:key:{}:{}", api_key_id, window_start);

        // Increment counter
        let count: u64 = conn.incr(&key, 1).await?;

        // Set expiry on first request
        if count == 1 {
            let _: () = conn.expire(&key, window as i64 + 1).await?;
        }

        if count > limit {
            // Rate limited - calculate retry after
            let retry_after = window - (now % window);
            Ok(RateLimitResult::RetryAfter(retry_after))
        } else {
            Ok(RateLimitResult::Allowed)
        }
    }

    /// Check user-level rate limit (aggregate across all keys)
    pub async fn check_user_limit(
        &self,
        user_id: uuid::Uuid,
        requests_per_hour: u64,
    ) -> Result<RateLimitResult, redis::RedisError> {
        if requests_per_hour == 0 {
            return Ok(RateLimitResult::Allowed);
        }

        let mut conn = self.redis.get_multiplexed_async_connection().await?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Window is 1 hour
        let window = 3600u64;
        let window_start = now - (now % window);
        let key = format!("rate_limit:user:{}:{}", user_id, window_start);

        let count: u64 = conn.incr(&key, 1).await?;

        if count == 1 {
            let _: () = conn.expire(&key, window as i64 + 1).await?;
        }

        if count > requests_per_hour {
            let retry_after = window - (now % window);
            Ok(RateLimitResult::RetryAfter(retry_after))
        } else {
            Ok(RateLimitResult::Allowed)
        }
    }

    /// Check IP-based rate limit (for unauthenticated requests)
    pub async fn check_ip_limit(
        &self,
        ip: &str,
        requests_per_hour: u64,
    ) -> Result<RateLimitResult, redis::RedisError> {
        if requests_per_hour == 0 {
            return Ok(RateLimitResult::Allowed);
        }

        let mut conn = self.redis.get_multiplexed_async_connection().await?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Window is 1 hour
        let window = 3600u64;
        let window_start = now - (now % window);
        let key = format!("rate_limit:ip:{}:{}", ip, window_start);

        let count: u64 = conn.incr(&key, 1).await?;

        if count == 1 {
            let _: () = conn.expire(&key, window as i64 + 1).await?;
        }

        if count > requests_per_hour {
            let retry_after = window - (now % window);
            Ok(RateLimitResult::RetryAfter(retry_after))
        } else {
            Ok(RateLimitResult::Allowed)
        }
    }

    /// Check monthly file upload limit for a user
    pub async fn check_monthly_file_limit(
        &self,
        user_id: uuid::Uuid,
        max_files: Option<i32>,
    ) -> Result<RateLimitResult, redis::RedisError> {
        let max = match max_files {
            Some(m) => m as u64,
            None => return Ok(RateLimitResult::Allowed),
        };

        let mut conn = self.redis.get_multiplexed_async_connection().await?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Get current month (approximate with 30-day windows)
        let month = now / (30 * 24 * 3600);
        let key = format!("rate_limit:files:{}:{}", user_id, month);

        let count: u64 = conn.incr(&key, 1).await?;

        // Expire after 35 days to be safe
        if count == 1 {
            let _: () = conn.expire(&key, 35 * 24 * 3600).await?;
        }

        if count > max {
            // Calculate seconds until next month
            let next_month = (month + 1) * 30 * 24 * 3600;
            let retry_after = next_month - now;
            Ok(RateLimitResult::RetryAfter(retry_after))
        } else {
            Ok(RateLimitResult::Allowed)
        }
    }

    /// Increment file count for a user (call after successful upload)
    pub async fn increment_file_count(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<u64, redis::RedisError> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let month = now / (30 * 24 * 3600);
        let key = format!("rate_limit:files:{}:{}", user_id, month);

        let count: u64 = conn.incr(&key, 1).await?;

        if count == 1 {
            let _: () = conn.expire(&key, 35 * 24 * 3600).await?;
        }

        Ok(count)
    }

    /// Get current count for a rate limit key (for debugging/metrics)
    pub async fn get_current_count(&self, key: &str) -> Result<u64, redis::RedisError> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        let count: u64 = conn.get(key).await.unwrap_or(0);
        Ok(count)
    }

    /// Reset rate limit for a key (useful for testing or admin operations)
    pub async fn reset_limit(&self, key: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        conn.del::<_, ()>(key).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_result_display() {
        assert_eq!(format!("{:?}", RateLimitResult::Allowed), "Allowed");
        assert_eq!(
            format!("{:?}", RateLimitResult::RetryAfter(60)),
            "RetryAfter(60)"
        );
    }
}
