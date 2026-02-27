//! PII Redacta API Library
//!
//! REST API for PII detection and redaction.

pub mod handlers;
pub mod middleware;

#[cfg(test)]
mod security_test;

use axum::{
    middleware::{from_fn, map_response},
    routing::{get, post},
    Router,
};
use handlers::JobQueue;
use std::sync::Arc;

/// Create the API router
pub fn create_app() -> Router {
    let job_queue = Arc::new(JobQueue::new());

    Router::new()
        .route("/health", get(handlers::health::health))
        .route("/metrics", get(handlers::metrics::metrics))
        .route("/api/v1/detect", post(handlers::detection::detect))
        .route("/api/v1/upload", post(handlers::upload::upload))
        .route("/api/v1/jobs/:job_id", get(handlers::jobs::get_job_status))
        .layer(map_response(middleware::security::security_headers))
        .layer(from_fn(middleware::security::limit_body_size))
        .with_state(job_queue)
}
