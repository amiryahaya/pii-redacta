//! Metrics handler for Prometheus

use axum::extract::State;
use axum::http::StatusCode;
use std::sync::Arc;

use super::JobQueue;
use crate::AppState;

/// Simple Prometheus-compatible metrics endpoint (MVP — static zeros)
pub async fn metrics(State(_queue): State<Arc<JobQueue>>) -> (StatusCode, String) {
    let output = format!(
        r#"# HELP pii_detection_requests_total Total number of detection requests
# TYPE pii_detection_requests_total counter
pii_detection_requests_total {}

# HELP pii_entities_detected_total Total entities detected
# TYPE pii_entities_detected_total counter
pii_entities_detected_total {}

# HELP pii_files_uploaded_total Total files uploaded
# TYPE pii_files_uploaded_total counter
pii_files_uploaded_total {}
"#,
        0, 0, 0
    );

    (StatusCode::OK, output)
}

/// Authenticated Prometheus metrics endpoint with real counters
pub async fn metrics_authenticated(State(state): State<AppState>) -> (StatusCode, String) {
    (StatusCode::OK, state.metrics.render_prometheus())
}

#[cfg(test)]
#[path = "metrics_test.rs"]
mod tests;
