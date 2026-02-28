//! Usage logging for PII Redacta
//!
//! Inserts records into the `usage_logs` table for analytics and quota tracking.

use sqlx::PgPool;
use uuid::Uuid;

/// Parameters for recording a usage log entry.
pub struct UsageRecord<'a> {
    pub user_id: Uuid,
    pub api_key_id: Option<Uuid>,
    pub request_type: &'a str,
    pub file_name: Option<&'a str>,
    pub file_size_bytes: Option<i32>,
    pub file_type: Option<&'a str>,
    pub processing_time_ms: Option<i32>,
    pub page_count: Option<i32>,
    pub detections_count: Option<i32>,
    pub success: bool,
    pub error_message: Option<&'a str>,
    pub ip_address: Option<&'a str>,
}

/// Record a usage log entry and return the generated row ID.
pub async fn record_usage(pool: &PgPool, record: &UsageRecord<'_>) -> Result<Uuid, sqlx::Error> {
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO usage_logs (
            user_id, api_key_id, request_type,
            file_name, file_size_bytes, file_type,
            processing_time_ms, page_count, detections_count,
            success, error_message, ip_address
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12::inet)
        RETURNING id
        "#,
    )
    .bind(record.user_id)
    .bind(record.api_key_id)
    .bind(record.request_type)
    .bind(record.file_name)
    .bind(record.file_size_bytes)
    .bind(record.file_type)
    .bind(record.processing_time_ms)
    .bind(record.page_count)
    .bind(record.detections_count)
    .bind(record.success)
    .bind(record.error_message)
    .bind(record.ip_address)
    .fetch_one(pool)
    .await?;

    Ok(id)
}
