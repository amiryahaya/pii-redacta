//! Application metrics registry using atomic counters
//!
//! Provides Prometheus-compatible metrics without external crate dependencies.

use std::sync::atomic::{AtomicU64, Ordering};

/// Application-wide metrics registry
pub struct AppMetrics {
    pub detection_requests: AtomicU64,
    pub entities_detected: AtomicU64,
    pub files_uploaded: AtomicU64,
    pub jobs_completed: AtomicU64,
    pub jobs_failed: AtomicU64,
    /// Cumulative detection duration in microseconds (for computing averages)
    detection_duration_us: AtomicU64,
}

impl AppMetrics {
    pub fn new() -> Self {
        Self {
            detection_requests: AtomicU64::new(0),
            entities_detected: AtomicU64::new(0),
            files_uploaded: AtomicU64::new(0),
            jobs_completed: AtomicU64::new(0),
            jobs_failed: AtomicU64::new(0),
            detection_duration_us: AtomicU64::new(0),
        }
    }

    /// Record a detection request with entity count and duration
    pub fn record_detection(&self, entity_count: u64, duration_ms: f64) {
        self.detection_requests.fetch_add(1, Ordering::Relaxed);
        self.entities_detected
            .fetch_add(entity_count, Ordering::Relaxed);
        let duration_us = (duration_ms * 1000.0) as u64;
        self.detection_duration_us
            .fetch_add(duration_us, Ordering::Relaxed);
    }

    /// Record a file upload
    pub fn record_upload(&self) {
        self.files_uploaded.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a completed job
    pub fn record_job_completed(&self) {
        self.jobs_completed.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a failed job
    pub fn record_job_failed(&self) {
        self.jobs_failed.fetch_add(1, Ordering::Relaxed);
    }

    /// Render metrics in Prometheus text exposition format
    pub fn render_prometheus(&self) -> String {
        let requests = self.detection_requests.load(Ordering::Relaxed);
        let entities = self.entities_detected.load(Ordering::Relaxed);
        let files = self.files_uploaded.load(Ordering::Relaxed);
        let jobs_ok = self.jobs_completed.load(Ordering::Relaxed);
        let jobs_fail = self.jobs_failed.load(Ordering::Relaxed);
        let duration_us = self.detection_duration_us.load(Ordering::Relaxed);
        let duration_secs = duration_us as f64 / 1_000_000.0;

        format!(
            r#"# HELP pii_detection_requests_total Total number of detection requests
# TYPE pii_detection_requests_total counter
pii_detection_requests_total {}

# HELP pii_entities_detected_total Total entities detected
# TYPE pii_entities_detected_total counter
pii_entities_detected_total {}

# HELP pii_files_uploaded_total Total files uploaded
# TYPE pii_files_uploaded_total counter
pii_files_uploaded_total {}

# HELP pii_jobs_completed_total Total jobs completed successfully
# TYPE pii_jobs_completed_total counter
pii_jobs_completed_total {}

# HELP pii_jobs_failed_total Total jobs that failed
# TYPE pii_jobs_failed_total counter
pii_jobs_failed_total {}

# HELP pii_processing_duration_seconds_total Cumulative processing duration in seconds
# TYPE pii_processing_duration_seconds_total counter
pii_processing_duration_seconds_total {}
"#,
            requests, entities, files, jobs_ok, jobs_fail, duration_secs
        )
    }
}

impl Default for AppMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_new() {
        let m = AppMetrics::new();
        assert_eq!(m.detection_requests.load(Ordering::Relaxed), 0);
        assert_eq!(m.entities_detected.load(Ordering::Relaxed), 0);
        assert_eq!(m.files_uploaded.load(Ordering::Relaxed), 0);
        assert_eq!(m.jobs_completed.load(Ordering::Relaxed), 0);
        assert_eq!(m.jobs_failed.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_record_detection() {
        let m = AppMetrics::new();
        m.record_detection(3, 1.5);
        m.record_detection(2, 0.5);
        assert_eq!(m.detection_requests.load(Ordering::Relaxed), 2);
        assert_eq!(m.entities_detected.load(Ordering::Relaxed), 5);
    }

    #[test]
    fn test_record_upload() {
        let m = AppMetrics::new();
        m.record_upload();
        m.record_upload();
        assert_eq!(m.files_uploaded.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_record_jobs() {
        let m = AppMetrics::new();
        m.record_job_completed();
        m.record_job_completed();
        m.record_job_failed();
        assert_eq!(m.jobs_completed.load(Ordering::Relaxed), 2);
        assert_eq!(m.jobs_failed.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_render_prometheus() {
        let m = AppMetrics::new();
        m.record_detection(5, 2.0);
        m.record_upload();
        m.record_job_completed();

        let output = m.render_prometheus();
        assert!(output.contains("pii_detection_requests_total 1"));
        assert!(output.contains("pii_entities_detected_total 5"));
        assert!(output.contains("pii_files_uploaded_total 1"));
        assert!(output.contains("pii_jobs_completed_total 1"));
        assert!(output.contains("pii_jobs_failed_total 0"));
    }
}
