//! Redis connection pool wrapper
//!
//! Provides a thin async wrapper around the `redis` crate with
//! `MultiplexedConnection` for use with Tokio.

/// Error type for Redis operations
pub type RedisError = redis::RedisError;

/// Redis connection pool wrapper
#[derive(Clone)]
pub struct RedisPool {
    client: redis::Client,
}

impl RedisPool {
    /// Create a new Redis pool from a connection URL
    pub async fn new(url: &str) -> Result<Self, RedisError> {
        let client = redis::Client::open(url)?;
        // Verify connectivity
        let mut conn = client.get_multiplexed_async_connection().await?;
        let _: String = redis::cmd("PING").query_async(&mut conn).await?;
        Ok(Self { client })
    }

    /// Get an async multiplexed connection
    pub async fn get_connection(&self) -> Result<redis::aio::MultiplexedConnection, RedisError> {
        self.client.get_multiplexed_async_connection().await
    }

    /// Health check: PING Redis and verify PONG response
    pub async fn health_check(&self) -> Result<(), RedisError> {
        let mut conn = self.get_connection().await?;
        let _: String = redis::cmd("PING").query_async(&mut conn).await?;
        Ok(())
    }

    /// Set a key to an integer value with a TTL in seconds.
    pub async fn set_with_expiry(
        &self,
        key: &str,
        value: i64,
        ttl_secs: u64,
    ) -> Result<(), RedisError> {
        let mut conn = self.get_connection().await?;
        redis::cmd("SET")
            .arg(key)
            .arg(value)
            .arg("EX")
            .arg(ttl_secs)
            .query_async(&mut conn)
            .await
    }

    /// Get an integer value from a key, returning None if the key doesn't exist.
    pub async fn get_i64(&self, key: &str) -> Result<Option<i64>, RedisError> {
        let mut conn = self.get_connection().await?;
        redis::cmd("GET").arg(key).query_async(&mut conn).await
    }

    /// Increment a key and set expiry atomically via a Redis pipeline.
    ///
    /// Returns the new value after increment. Always sets TTL so even if
    /// a previous EXPIRE was missed, the key will eventually expire.
    pub async fn incr_with_expiry(&self, key: &str, ttl_secs: u64) -> Result<i64, RedisError> {
        let mut conn = self.get_connection().await?;
        // Use a pipeline to execute INCR and EXPIRE atomically (single round-trip).
        // EXPIRE is always sent (idempotent) to avoid a race where the key
        // exists without a TTL if a crash occurs between separate commands.
        let (count,): (i64,) = redis::pipe()
            .atomic()
            .incr(key, 1i64)
            .expire(key, ttl_secs as i64)
            .ignore()
            .query_async(&mut conn)
            .await?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_redis_error_display() {
        // Verify that RedisError (re-exported) has Display trait
        let err = redis::RedisError::from((redis::ErrorKind::TypeError, "test"));
        assert!(err.to_string().contains("test"));
    }
}
