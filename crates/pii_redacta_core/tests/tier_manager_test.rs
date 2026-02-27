//! TierManager Integration Tests
//!
//! These tests require PostgreSQL and Redis to be running.
//! Run with: cargo test --test tier_manager_test

use pii_redacta_core::db::tier_manager::TierManager;
use pii_redacta_core::db::Database;
use std::sync::Arc;
use std::time::Duration;

/// Get database URL from environment or use default
fn database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://pii_redacta:pii_redacta_dev@localhost:5432/pii_redacta_dev".to_string()
    })
}

/// Get Redis URL from environment or use default
fn redis_url() -> String {
    std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string())
}

/// Helper to create a test database connection
async fn setup_db() -> Arc<Database> {
    let db = Database::new(&database_url())
        .await
        .expect("Failed to connect to database");
    Arc::new(db)
}

/// Helper to create a TierManager with Redis
async fn setup_tier_manager() -> TierManager {
    let db = setup_db().await;
    TierManager::with_redis(db, &redis_url())
        .expect("Failed to create TierManager with Redis")
        .with_cache_ttl(Duration::from_secs(5)) // Short TTL for tests
}

/// Helper to create a TierManager without Redis
async fn setup_tier_manager_no_cache() -> TierManager {
    let db = setup_db().await;
    TierManager::new(db)
}

#[tokio::test]
async fn test_get_tier_by_name_trial() {
    let manager = setup_tier_manager().await;

    let tier = manager
        .get_by_name("trial")
        .await
        .expect("Should get trial tier");

    assert_eq!(tier.name, "trial");
    assert_eq!(tier.display_name, "Trial");
    assert!(tier.is_active);
    assert!(tier.is_public);
}

#[tokio::test]
async fn test_get_tier_by_name_starter() {
    let manager = setup_tier_manager().await;

    let tier = manager
        .get_by_name("starter")
        .await
        .expect("Should get starter tier");

    assert_eq!(tier.name, "starter");
    assert_eq!(tier.display_name, "Starter");
    assert_eq!(tier.monthly_price_cents, Some(999)); // $9.99
}

#[tokio::test]
async fn test_get_tier_by_name_pro() {
    let manager = setup_tier_manager().await;

    let tier = manager
        .get_by_name("pro")
        .await
        .expect("Should get pro tier");

    assert_eq!(tier.name, "pro");
    assert_eq!(tier.display_name, "Pro");
    assert_eq!(tier.monthly_price_cents, Some(4999)); // $49.99
}

#[tokio::test]
async fn test_get_tier_by_name_enterprise() {
    let manager = setup_tier_manager().await;

    let tier = manager
        .get_by_name("enterprise")
        .await
        .expect("Should get enterprise tier");

    assert_eq!(tier.name, "enterprise");
    assert_eq!(tier.display_name, "Enterprise");
    assert!(tier.monthly_price_cents.is_none()); // Contact sales
}

#[tokio::test]
async fn test_get_tier_by_name_not_found() {
    let manager = setup_tier_manager().await;

    let result = manager.get_by_name("nonexistent").await;

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("not found"));
}

#[tokio::test]
async fn test_get_tier_by_id() {
    let manager = setup_tier_manager().await;

    // First get by name to get the ID
    let tier_by_name = manager.get_by_name("trial").await.unwrap();

    // Now get by ID
    let tier_by_id = manager
        .get_by_id(tier_by_name.id)
        .await
        .expect("Should get tier by ID");

    assert_eq!(tier_by_id.id, tier_by_name.id);
    assert_eq!(tier_by_id.name, tier_by_name.name);
}

#[tokio::test]
async fn test_list_active_tiers() {
    let manager = setup_tier_manager().await;

    let tiers = manager
        .list_active_tiers()
        .await
        .expect("Should list tiers");

    // Should have 4 default tiers
    assert_eq!(tiers.len(), 4);

    // Should be ordered by sort_order
    assert_eq!(tiers[0].name, "trial");
    assert_eq!(tiers[1].name, "starter");
    assert_eq!(tiers[2].name, "pro");
    assert_eq!(tiers[3].name, "enterprise");
}

#[tokio::test]
async fn test_tier_limits_trial() {
    let manager = setup_tier_manager().await;

    let tier = manager.get_by_name("trial").await.unwrap();
    let limits = tier.limits.0;

    assert!(limits.api_enabled);
    assert_eq!(limits.max_api_keys, Some(2));
    assert_eq!(limits.max_file_size, Some(10_485_760)); // 10MB
    assert_eq!(limits.max_files_per_month, Some(50));
    assert_eq!(limits.max_pages_per_file, Some(100));
    assert_eq!(limits.max_total_size, Some(524_288_000)); // 500MB
    assert_eq!(limits.playground_max_daily, Some(5));
    assert_eq!(limits.retention_days, Some(7));
}

#[tokio::test]
async fn test_tier_limits_enterprise() {
    let manager = setup_tier_manager().await;

    let tier = manager.get_by_name("enterprise").await.unwrap();
    let limits = tier.limits.0;

    assert!(limits.api_enabled);
    assert!(limits.max_api_keys.is_none()); // Unlimited
    assert!(limits.max_file_size.is_none()); // Unlimited
    assert!(limits.max_files_per_month.is_none()); // Unlimited
}

#[tokio::test]
async fn test_tier_features_trial() {
    let manager = setup_tier_manager().await;

    let tier = manager.get_by_name("trial").await.unwrap();
    let features = tier.features.0;

    assert!(features.playground);
    assert!(!features.batch_processing);
    assert!(!features.custom_rules);
    assert!(!features.email_support);
    assert!(!features.webhooks);
    assert_eq!(features.rate_limit_per_minute, Some(10));
    assert!(features.sla.is_none());
}

#[tokio::test]
async fn test_tier_features_pro() {
    let manager = setup_tier_manager().await;

    let tier = manager.get_by_name("pro").await.unwrap();
    let features = tier.features.0;

    assert!(features.playground);
    assert!(features.batch_processing);
    assert!(features.custom_rules);
    assert!(features.email_support);
    assert!(features.webhooks);
    assert_eq!(features.rate_limit_per_minute, Some(100));
    assert_eq!(features.sla, Some("99.9%".to_string()));
}

#[tokio::test]
async fn test_tier_limits_file_size_check() {
    let manager = setup_tier_manager().await;

    let tier = manager.get_by_name("trial").await.unwrap();
    let limits = tier.limits.0;

    // 5MB file should be allowed
    assert!(limits.is_file_size_allowed(5_000_000));

    // 10MB file should be allowed (exactly at limit)
    assert!(limits.is_file_size_allowed(10_485_760));

    // 20MB file should NOT be allowed
    assert!(!limits.is_file_size_allowed(20_000_000));
}

#[tokio::test]
async fn test_tier_limits_storage_check() {
    let manager = setup_tier_manager().await;

    let tier = manager.get_by_name("trial").await.unwrap();
    let limits = tier.limits.0;

    // Currently at 100MB, adding 200MB should be fine (total 300MB < 500MB)
    assert!(limits.is_storage_available(100_000_000, 200_000_000));

    // Currently at 400MB, adding 200MB should NOT be fine (total 600MB > 500MB)
    assert!(!limits.is_storage_available(400_000_000, 200_000_000));
}

#[tokio::test]
async fn test_tier_limits_unlimited() {
    let manager = setup_tier_manager().await;

    let tier = manager.get_by_name("enterprise").await.unwrap();
    let limits = tier.limits.0;

    // Enterprise has no limits - any file size should be allowed
    assert!(limits.is_file_size_allowed(100_000_000_000)); // 100GB

    // Any storage should be available
    assert!(limits.is_storage_available(1_000_000_000_000, 1_000_000_000_000)); // 2TB
}

#[tokio::test]
async fn test_tier_features_has_feature() {
    let manager = setup_tier_manager().await;

    let tier = manager.get_by_name("starter").await.unwrap();
    let features = tier.features.0;

    assert!(features.has_feature("playground"));
    assert!(features.has_feature("email_support"));
    assert!(!features.has_feature("batch_processing"));
    assert!(!features.has_feature("custom_rules"));
    assert!(!features.has_feature("webhooks"));
    assert!(!features.has_feature("unknown_feature"));
}

#[tokio::test]
async fn test_cache_hit() {
    let manager = setup_tier_manager().await;

    // First call - should hit database
    let tier1 = manager.get_by_name("trial").await.unwrap();

    // Second call - should hit cache
    let tier2 = manager.get_by_name("trial").await.unwrap();

    assert_eq!(tier1.id, tier2.id);
    assert_eq!(tier1.name, tier2.name);
}

#[tokio::test]
async fn test_cache_invalidation() {
    let manager = setup_tier_manager().await;

    // Get tier to populate cache
    let tier = manager.get_by_name("trial").await.unwrap();

    // Invalidate cache
    manager
        .invalidate_cache(tier.id, Some(&tier.name))
        .await
        .expect("Should invalidate cache");

    // Get again - should still work (from database)
    let tier_after = manager.get_by_name("trial").await.unwrap();
    assert_eq!(tier.id, tier_after.id);
}

#[tokio::test]
async fn test_no_cache_mode() {
    let manager = setup_tier_manager_no_cache().await;

    // Should still work without Redis
    let tier = manager
        .get_by_name("trial")
        .await
        .expect("Should get tier without cache");
    assert_eq!(tier.name, "trial");

    let tiers = manager
        .list_active_tiers()
        .await
        .expect("Should list tiers without cache");
    assert_eq!(tiers.len(), 4);
}

#[tokio::test]
async fn test_get_by_id_not_found() {
    let manager = setup_tier_manager().await;

    let fake_id = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440999").unwrap();
    let result = manager.get_by_id(fake_id).await;

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("not found"));
}

#[tokio::test]
async fn test_tier_id_and_name_consistency() {
    let manager = setup_tier_manager().await;

    // Get tier by name
    let by_name = manager.get_by_name("pro").await.unwrap();

    // Get same tier by ID
    let by_id = manager.get_by_id(by_name.id).await.unwrap();

    // Both should have same data
    assert_eq!(by_name.id, by_id.id);
    assert_eq!(by_name.name, by_id.name);
    assert_eq!(by_name.display_name, by_id.display_name);
    assert_eq!(by_name.monthly_price_cents, by_id.monthly_price_cents);

    // Limits should match
    assert_eq!(by_name.limits.0.max_api_keys, by_id.limits.0.max_api_keys);
    assert_eq!(by_name.limits.0.max_file_size, by_id.limits.0.max_file_size);
}
