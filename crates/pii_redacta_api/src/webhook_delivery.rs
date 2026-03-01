//! Webhook delivery service
//!
//! Sprint 14: Background webhook delivery with retry logic and HMAC signing.

use hmac::{Hmac, Mac};
use pii_redacta_core::db::Database;
use sha2::Sha256;
use std::sync::Arc;

type HmacSha256 = Hmac<Sha256>;

/// Maximum consecutive failures before auto-disabling a webhook endpoint
const MAX_FAILURE_COUNT: i32 = 10;

/// Deliver a webhook payload to a URL with HMAC signature.
///
/// Returns the HTTP status code on success, or an error message on failure.
pub async fn deliver_webhook(
    url: &str,
    secret: &str,
    delivery_id: uuid::Uuid,
    event_type: &str,
    payload: &serde_json::Value,
) -> Result<u16, String> {
    let body = serde_json::to_string(payload).map_err(|e| format!("JSON serialize error: {e}"))?;

    // Compute HMAC-SHA256 signature
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).map_err(|e| format!("HMAC error: {e}"))?;
    mac.update(body.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("X-Webhook-Id", delivery_id.to_string())
        .header("X-Webhook-Event", event_type)
        .header("X-Webhook-Signature", format!("sha256={signature}"))
        .body(body)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    let status = response.status().as_u16();

    if (200..300).contains(&status) {
        Ok(status)
    } else {
        Err(format!("HTTP {status}"))
    }
}

/// Background delivery service that polls for pending deliveries and retries failed ones.
pub struct WebhookDeliveryService {
    db: Arc<Database>,
}

impl WebhookDeliveryService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Start the background delivery loop. Returns a `JoinHandle`.
    pub fn start(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                if let Err(e) = self.process_pending().await {
                    tracing::warn!(error = %e, "Webhook delivery poll error");
                }
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            }
        })
    }

    async fn process_pending(&self) -> Result<(), sqlx::Error> {
        let rows =
            sqlx::query_as::<_, (uuid::Uuid, uuid::Uuid, String, serde_json::Value, i32, i32)>(
                r#"
            SELECT d.id, d.endpoint_id, d.event_type, d.payload, d.attempts, d.max_attempts
            FROM webhook_deliveries d
            WHERE d.status = 'pending'
            AND (d.next_retry_at IS NULL OR d.next_retry_at <= NOW())
            ORDER BY d.created_at
            LIMIT 50
            "#,
            )
            .fetch_all(self.db.pool())
            .await?;

        for (delivery_id, endpoint_id, event_type, payload, attempts, max_attempts) in rows {
            // Load endpoint
            let endpoint = sqlx::query_as::<_, (String, String, bool)>(
                "SELECT url, secret, is_active FROM webhook_endpoints WHERE id = $1",
            )
            .bind(endpoint_id)
            .fetch_optional(self.db.pool())
            .await?;

            let (url, secret, is_active) = match endpoint {
                Some(ep) => ep,
                None => {
                    // Endpoint deleted; mark delivery as failed
                    let _ = sqlx::query(
                        "UPDATE webhook_deliveries SET status = 'failed', response_body = 'Endpoint deleted' WHERE id = $1",
                    )
                    .bind(delivery_id)
                    .execute(self.db.pool())
                    .await;
                    continue;
                }
            };

            if !is_active {
                let _ = sqlx::query(
                    "UPDATE webhook_deliveries SET status = 'failed', response_body = 'Endpoint disabled' WHERE id = $1",
                )
                .bind(delivery_id)
                .execute(self.db.pool())
                .await;
                continue;
            }

            let new_attempts = attempts + 1;

            match deliver_webhook(&url, &secret, delivery_id, &event_type, &payload).await {
                Ok(status_code) => {
                    // Success
                    let _ = sqlx::query(
                        r#"
                        UPDATE webhook_deliveries
                        SET status = 'delivered', http_status = $1, attempts = $2, delivered_at = NOW()
                        WHERE id = $3
                        "#,
                    )
                    .bind(status_code as i32)
                    .bind(new_attempts)
                    .bind(delivery_id)
                    .execute(self.db.pool())
                    .await;

                    // Reset failure count on success
                    let _ = sqlx::query(
                        "UPDATE webhook_endpoints SET failure_count = 0, last_triggered_at = NOW(), updated_at = NOW() WHERE id = $1",
                    )
                    .bind(endpoint_id)
                    .execute(self.db.pool())
                    .await;
                }
                Err(err) => {
                    let err_msg: String = err.chars().take(1000).collect();

                    if new_attempts >= max_attempts {
                        // Max retries exhausted
                        let _ = sqlx::query(
                            r#"
                            UPDATE webhook_deliveries
                            SET status = 'failed', response_body = $1, attempts = $2
                            WHERE id = $3
                            "#,
                        )
                        .bind(&err_msg)
                        .bind(new_attempts)
                        .bind(delivery_id)
                        .execute(self.db.pool())
                        .await;
                    } else {
                        // Schedule retry with exponential backoff
                        let backoff_secs = match new_attempts {
                            1 => 10,
                            2 => 60,
                            _ => 300,
                        };
                        let _ = sqlx::query(
                            r#"
                            UPDATE webhook_deliveries
                            SET attempts = $1, response_body = $2,
                                next_retry_at = NOW() + INTERVAL '1 second' * $3
                            WHERE id = $4
                            "#,
                        )
                        .bind(new_attempts)
                        .bind(&err_msg)
                        .bind(backoff_secs)
                        .bind(delivery_id)
                        .execute(self.db.pool())
                        .await;
                    }

                    // Increment failure count; auto-disable after threshold
                    let _ = sqlx::query(
                        r#"
                        UPDATE webhook_endpoints
                        SET failure_count = failure_count + 1,
                            is_active = CASE WHEN failure_count + 1 >= $1 THEN false ELSE is_active END,
                            updated_at = NOW()
                        WHERE id = $2
                        "#,
                    )
                    .bind(MAX_FAILURE_COUNT)
                    .bind(endpoint_id)
                    .execute(self.db.pool())
                    .await;
                }
            }
        }

        Ok(())
    }
}
