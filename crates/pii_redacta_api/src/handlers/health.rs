//! Health check handler
//!
//! Provides health check endpoint with dependency verification.

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::AppState;

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<DependenciesStatus>,
}

/// Dependency health status
#[derive(Debug, Serialize, Deserialize)]
pub struct DependenciesStatus {
    pub database: DependencyHealth,
    pub redis: Option<DependencyHealth>,
}

/// Single dependency health
#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyHealth {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Basic health check (no dependencies)
pub async fn health() -> (StatusCode, Json<HealthResponse>) {
    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            dependencies: None,
        }),
    )
}

/// Deep health check with dependency verification
///
/// Note (S9-R4-12): Requires `AppState` — only usable with `create_app_with_auth()`,
/// not the MVP `create_app()` which uses `Arc<JobQueue>` as state.
pub async fn health_deep(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    let start = std::time::Instant::now();

    // Check database connectivity
    let db_health = check_database(&state).await;

    // Check Redis connectivity (if configured)
    let redis_health = check_redis(&state).await;

    let all_healthy = db_health.status == "healthy"
        && redis_health
            .as_ref()
            .map_or(true, |r| r.status == "healthy");

    let response = HealthResponse {
        status: if all_healthy {
            "healthy".to_string()
        } else {
            "degraded".to_string()
        },
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        dependencies: Some(DependenciesStatus {
            database: db_health,
            redis: redis_health,
        }),
    };

    let status = if all_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    tracing::debug!(
        latency_ms = start.elapsed().as_millis(),
        "Health check completed"
    );

    (status, Json(response))
}

/// Check database connectivity
async fn check_database(state: &AppState) -> DependencyHealth {
    let start = std::time::Instant::now();

    match sqlx::query("SELECT 1").fetch_one(state.db.pool()).await {
        Ok(_) => DependencyHealth {
            status: "healthy".to_string(),
            latency_ms: Some(start.elapsed().as_millis() as u64),
            error: None,
        },
        Err(e) => {
            tracing::error!(error = %e, "Database health check failed");
            DependencyHealth {
                status: "unhealthy".to_string(),
                latency_ms: Some(start.elapsed().as_millis() as u64),
                error: Some(e.to_string()),
            }
        }
    }
}

/// Check Redis connectivity
async fn check_redis(state: &AppState) -> Option<DependencyHealth> {
    let redis = state.redis.as_ref()?;
    let start = std::time::Instant::now();

    match redis.health_check().await {
        Ok(()) => Some(DependencyHealth {
            status: "healthy".to_string(),
            latency_ms: Some(start.elapsed().as_millis() as u64),
            error: None,
        }),
        Err(e) => {
            tracing::error!(error = %e, "Redis health check failed");
            Some(DependencyHealth {
                status: "unhealthy".to_string(),
                latency_ms: Some(start.elapsed().as_millis() as u64),
                error: Some(e.to_string()),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_basic() {
        let (status, response) = health().await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(response.status, "healthy");
        assert!(!response.version.is_empty());
        assert!(response.dependencies.is_none());
    }

    #[test]
    fn test_health_response_serialization() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            version: "0.1.0".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            dependencies: Some(DependenciesStatus {
                database: DependencyHealth {
                    status: "healthy".to_string(),
                    latency_ms: Some(5),
                    error: None,
                },
                redis: None,
            }),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("healthy"));
        assert!(json.contains("0.1.0"));
    }
}
