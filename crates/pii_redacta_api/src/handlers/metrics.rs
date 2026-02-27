//! Metrics handler for Prometheus

use axum::http::StatusCode;

/// Simple Prometheus-compatible metrics endpoint
///
/// In production, this would use a proper metrics crate like `metrics`
/// and export counters, gauges, and histograms.
pub async fn metrics() -> (StatusCode, String) {
    let output = format!(
        r#"# HELP pii_detection_requests_total Total number of detection requests
# TYPE pii_detection_requests_total counter
pii_detection_requests_total {}

# HELP pii_processing_duration_seconds Processing duration in seconds
# TYPE pii_processing_duration_seconds histogram
pii_processing_duration_seconds_bucket{{le="0.001"}} {}
pii_processing_duration_seconds_bucket{{le="0.01"}} {}
pii_processing_duration_seconds_bucket{{le="0.1"}} {}
pii_processing_duration_seconds_bucket{{le="+Inf"}} {}
pii_processing_duration_seconds_sum {}
pii_processing_duration_seconds_count {}

# HELP pii_entities_detected_total Total entities detected
# TYPE pii_entities_detected_total counter
pii_entities_detected_total {}

# HELP pii_files_uploaded_total Total files uploaded
# TYPE pii_files_uploaded_total counter
pii_files_uploaded_total {}
"#,
        0, 0, 0, 0, 0, 0.0, 0, 0, 0
    );

    (StatusCode::OK, output)
}

#[cfg(test)]
#[path = "metrics_test.rs"]
mod tests;
