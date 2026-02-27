//! PII Redacta API Library
//!
//! REST API for PII detection and redaction.

pub mod handlers;

use axum::{
    routing::{get, post},
    Router,
};
use pii_redacta_core::detection::PatternDetector;
use std::sync::Arc;

/// Create the API router
pub fn create_app() -> Router {
    let detector = Arc::new(PatternDetector::new());

    Router::new()
        .route("/health", get(handlers::health::health))
        .route("/api/v1/detect", post(handlers::detection::detect))
        .with_state(detector)
}
