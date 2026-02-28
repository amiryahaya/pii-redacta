//! Database models and access layer for PII Redacta
//!
//! This module provides:
//! - Database connection pool management
//! - Data models (Tier, User, Subscription, etc.)
//! - Repository patterns for database access

pub mod api_key_manager;
pub mod models;
pub mod redis;
pub mod tier_manager;
pub mod usage;

use sqlx::{Pool, Postgres};
use std::sync::Arc;

/// Database connection pool wrapper
#[derive(Clone)]
pub struct Database {
    pool: Pool<Postgres>,
}

impl Database {
    /// Create a new database connection pool
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &Pool<Postgres> {
        &self.pool
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<(), sqlx::migrate::MigrateError> {
        sqlx::migrate!("./migrations").run(&self.pool).await
    }
}

/// Shared database handle
pub type DbPool = Arc<Database>;
