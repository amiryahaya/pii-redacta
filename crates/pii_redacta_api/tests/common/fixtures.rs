#![allow(dead_code)]
//! Test fixtures and factories for integration tests
//!
//! Provides helper functions to create test data in the database.

use pii_redacta_core::db::models::{SubscriptionStatus, TierFeatures, TierLimits};
use pii_redacta_core::db::Database;
use uuid::Uuid;

/// Create a test user
pub async fn create_user(
    db: &Database,
    email: &str,
    password_hash: Option<&str>,
) -> Result<Uuid, sqlx::Error> {
    let user_id = Uuid::new_v4();
    let hash = password_hash.unwrap_or("$argon2id$v=19$m=19456,t=2,p=1$c2FsdHNhbHRzYWx0$hash");

    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, created_at, updated_at)
        VALUES ($1, $2, $3, NOW(), NOW())
        "#,
    )
    .bind(user_id)
    .bind(email)
    .bind(hash)
    .execute(db.pool())
    .await?;

    Ok(user_id)
}

/// Create a test user with profile information
pub async fn create_user_with_profile(
    db: &Database,
    email: &str,
    display_name: Option<&str>,
    company_name: Option<&str>,
) -> Result<Uuid, sqlx::Error> {
    let user_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, display_name, company_name, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
        "#,
    )
    .bind(user_id)
    .bind(email)
    .bind("$argon2id$v=19$m=19456,t=2,p=1$c2FsdHNhbHRzYWx0$hash")
    .bind(display_name)
    .bind(company_name)
    .execute(db.pool())
    .await?;

    Ok(user_id)
}

/// Create a test tier
pub async fn create_tier(
    db: &Database,
    name: &str,
    display_name: &str,
    limits: TierLimits,
    features: TierFeatures,
) -> Result<Uuid, sqlx::Error> {
    let tier_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO tiers (
            id, name, display_name, description, limits, features,
            monthly_price_cents, yearly_price_cents, is_public, is_active, sort_order,
            created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW(), NOW())
        "#,
    )
    .bind(tier_id)
    .bind(name)
    .bind(display_name)
    .bind(format!("{} tier description", name))
    .bind(sqlx::types::Json(limits))
    .bind(sqlx::types::Json(features))
    .bind(Some(0i32)) // Free tier
    .bind(None::<i32>)
    .bind(true) // is_public
    .bind(true) // is_active
    .bind(0i32) // sort_order
    .execute(db.pool())
    .await?;

    Ok(tier_id)
}

/// Create a test subscription
pub async fn create_subscription(
    db: &Database,
    user_id: Uuid,
    tier_id: Uuid,
    status: SubscriptionStatus,
) -> Result<Uuid, sqlx::Error> {
    let subscription_id = Uuid::new_v4();

    let (period_start, period_end) = match status {
        SubscriptionStatus::Trial => {
            let start = chrono::Utc::now();
            let end = start + chrono::Duration::days(14);
            (Some(start), Some(end))
        }
        SubscriptionStatus::Active => {
            let start = chrono::Utc::now();
            let end = start + chrono::Duration::days(30);
            (Some(start), Some(end))
        }
        _ => (None, None),
    };

    sqlx::query(
        r#"
        INSERT INTO subscriptions (
            id, user_id, tier_id, status, current_period_start, current_period_end,
            cancel_at_period_end, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
        "#,
    )
    .bind(subscription_id)
    .bind(user_id)
    .bind(tier_id)
    .bind(status)
    .bind(period_start)
    .bind(period_end)
    .bind(false)
    .execute(db.pool())
    .await?;

    Ok(subscription_id)
}

/// Create a complete test user with subscription
pub async fn create_user_with_subscription(
    db: &Database,
    email: &str,
    tier_name: &str,
) -> Result<(Uuid, Uuid, Uuid), sqlx::Error> {
    // Create user
    let user_id = create_user(db, email, None).await?;

    // Create tier
    let limits = TierLimits {
        api_enabled: true,
        max_api_keys: Some(5),
        max_file_size: Some(10_485_760),
        max_files_per_month: Some(100),
        max_pages_per_file: Some(50),
        max_total_size: Some(524_288_000),
        playground_max_daily: Some(10),
        playground_max_file_size: Some(1_048_576),
        retention_days: Some(30),
    };

    let features = TierFeatures {
        batch_processing: true,
        custom_rules: false,
        email_support: true,
        playground: true,
        rate_limit_per_minute: Some(60),
        sla: Some("99%".to_string()),
        webhooks: false,
    };

    let tier_id = create_tier(
        db,
        tier_name,
        &format!("Test {}", tier_name),
        limits,
        features,
    )
    .await?;

    // Create subscription
    let subscription_id =
        create_subscription(db, user_id, tier_id, SubscriptionStatus::Trial).await?;

    Ok((user_id, tier_id, subscription_id))
}

/// Create a test API key
pub async fn create_api_key(
    db: &Database,
    user_id: Uuid,
    name: &str,
    key_prefix: &str,
    key_hash: &str,
) -> Result<Uuid, sqlx::Error> {
    let key_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO api_keys (
            id, user_id, key_prefix, key_hash, name,
            is_active, created_at
        ) VALUES ($1, $2, $3, $4, $5, true, NOW())
        "#,
    )
    .bind(key_id)
    .bind(user_id)
    .bind(key_prefix)
    .bind(key_hash)
    .bind(name)
    .execute(db.pool())
    .await?;

    Ok(key_id)
}

/// Get the trial tier ID
pub async fn get_trial_tier_id(db: &Database) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as("SELECT id FROM tiers WHERE name = 'trial'")
        .fetch_one(db.pool())
        .await?;

    Ok(row.0)
}

/// Clean up test data
pub async fn cleanup_test_data(db: &Database, user_ids: &[Uuid]) {
    for user_id in user_ids {
        // Delete usage logs
        let _ = sqlx::query("DELETE FROM usage_logs WHERE user_id = $1")
            .bind(user_id)
            .execute(db.pool())
            .await;

        // Delete API keys
        let _ = sqlx::query("DELETE FROM api_keys WHERE user_id = $1")
            .bind(user_id)
            .execute(db.pool())
            .await;

        // Delete subscriptions
        let _ = sqlx::query("DELETE FROM subscriptions WHERE user_id = $1")
            .bind(user_id)
            .execute(db.pool())
            .await;

        // Delete user
        let _ = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(db.pool())
            .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common;

    #[tokio::test]
    async fn test_create_user() {
        let db = common::setup_db().await;
        let email = format!("test-{:.8}@example.com", Uuid::new_v4());

        let user_id = create_user(&db, &email, None)
            .await
            .expect("Failed to create user");
        assert_ne!(user_id, Uuid::nil());

        // Clean up
        cleanup_test_data(&db, &[user_id]).await;
    }

    #[tokio::test]
    async fn test_create_tier() {
        let db = common::setup_db().await;

        let limits = TierLimits::default();
        let features = TierFeatures::default();

        // Use a unique tier name to avoid conflicts
        let tier_name = format!("test-tier-{:.8}", Uuid::new_v4());
        let tier_id = create_tier(&db, &tier_name, "Test Tier", limits, features)
            .await
            .expect("Failed to create tier");

        assert_ne!(tier_id, Uuid::nil());

        // Clean up
        let _ = sqlx::query("DELETE FROM tiers WHERE id = $1")
            .bind(tier_id)
            .execute(db.pool())
            .await;
    }

    #[tokio::test]
    async fn test_get_trial_tier_id() {
        let db = common::setup_db().await;
        let tier_id = get_trial_tier_id(&db)
            .await
            .expect("Failed to get trial tier");
        assert_ne!(tier_id, Uuid::nil());
    }
}
