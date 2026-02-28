//! In-memory IP-based rate limiter
//!
//! Lightweight rate limiter that works without Redis, suitable for
//! single-instance deployments and development/test environments.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

/// In-memory rate limiter using a sliding window per IP.
pub struct InMemoryRateLimiter {
    /// IP → (request count, window start)
    state: Mutex<HashMap<String, (u64, Instant)>>,
}

impl InMemoryRateLimiter {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(HashMap::new()),
        }
    }

    /// Check whether `ip` is within the rate limit.
    /// Returns `true` if the request is allowed, `false` if it should be rejected.
    pub fn check_ip(&self, ip: &str, max_requests: u64, window_secs: u64) -> bool {
        let mut map = self.state.lock().expect("rate limiter mutex poisoned");
        let now = Instant::now();

        // Evict expired entries when the map grows too large to prevent unbounded memory
        if map.len() > Self::EVICTION_THRESHOLD {
            map.retain(|_, (_, start)| now.duration_since(*start).as_secs() < window_secs * 2);
        }

        let entry = map.entry(ip.to_string()).or_insert((0, now));

        // Reset window if expired
        if now.duration_since(entry.1).as_secs() >= window_secs {
            entry.0 = 0;
            entry.1 = now;
        }

        entry.0 += 1;
        entry.0 <= max_requests
    }

    /// Eviction runs when the map exceeds this many entries.
    const EVICTION_THRESHOLD: usize = 10_000;
}

impl Default for InMemoryRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allows_within_limit() {
        let limiter = InMemoryRateLimiter::new();
        for _ in 0..10 {
            assert!(limiter.check_ip("127.0.0.1", 10, 60));
        }
    }

    #[test]
    fn test_rejects_over_limit() {
        let limiter = InMemoryRateLimiter::new();
        for _ in 0..10 {
            limiter.check_ip("127.0.0.1", 10, 60);
        }
        assert!(!limiter.check_ip("127.0.0.1", 10, 60));
    }

    #[test]
    fn test_separate_ips() {
        let limiter = InMemoryRateLimiter::new();
        for _ in 0..10 {
            limiter.check_ip("1.2.3.4", 10, 60);
        }
        // Different IP should still be allowed
        assert!(limiter.check_ip("5.6.7.8", 10, 60));
        // Original IP should be blocked
        assert!(!limiter.check_ip("1.2.3.4", 10, 60));
    }
}
