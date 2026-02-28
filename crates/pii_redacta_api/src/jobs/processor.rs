//! Background job processor
//!
//! Polls the in-memory job queue for pending jobs, processes them through
//! the extraction and detection pipeline, and records results.

use crate::handlers::{JobQueue, JobResult, JobStatus};
use crate::metrics::AppMetrics;
use pii_redacta_core::detection::PatternDetector;
use pii_redacta_core::extraction::Extractor;
use std::sync::Arc;
use tracing::{debug, error, info};

/// Background job processor that runs as a spawned tokio task
pub struct JobProcessor {
    queue: Arc<JobQueue>,
    metrics: Option<Arc<AppMetrics>>,
}

impl JobProcessor {
    pub fn new(queue: Arc<JobQueue>, metrics: Option<Arc<AppMetrics>>) -> Self {
        Self { queue, metrics }
    }

    /// Start the processor loop. Returns a JoinHandle for the spawned task.
    pub fn start(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            info!("Job processor started");
            loop {
                match self.queue.get_pending() {
                    Some(job_id) => {
                        debug!(job_id = %job_id, "Processing job");
                        self.process_job(&job_id);
                    }
                    None => {
                        // No pending jobs — sleep before polling again
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    }
                }
            }
        })
    }

    fn process_job(&self, job_id: &str) {
        // Retrieve the job content (cloned to release the lock quickly)
        let (content, mime_type) = match self.queue.get(job_id) {
            Some(job) => (job.content.clone(), job.mime_type.clone()),
            None => {
                error!(job_id = %job_id, "Job not found during processing");
                return;
            }
        };

        let start = std::time::Instant::now();

        // Step 1: Extract text from file content
        let extracted = match Extractor::extract(&content, Some(&mime_type)) {
            Ok(doc) => doc,
            Err(e) => {
                let err_msg = format!("Extraction failed: {}", e);
                error!(job_id = %job_id, error = %err_msg, "Job extraction failed");
                self.queue
                    .update_job(job_id, JobStatus::Failed, None, Some(err_msg));
                if let Some(ref metrics) = self.metrics {
                    metrics.record_job_failed();
                }
                return;
            }
        };

        // Step 2: Detect PII entities
        let detector = PatternDetector::new();
        let entities = detector.detect_all(&extracted.text);

        let processing_time_ms = start.elapsed().as_secs_f64() * 1000.0;

        // Step 3: Build result and mark completed
        let result = JobResult {
            entities,
            processing_time_ms,
            redacted_text: None,
            extracted_text_length: extracted.text.len(),
        };

        info!(
            job_id = %job_id,
            entities_found = result.entities.len(),
            processing_time_ms = %format!("{:.2}", processing_time_ms),
            "Job completed"
        );

        self.queue
            .update_job(job_id, JobStatus::Completed, Some(result), None);

        if let Some(ref metrics) = self.metrics {
            metrics.record_job_completed();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::Job;

    #[tokio::test]
    async fn test_process_text_job() {
        let queue = Arc::new(JobQueue::new());
        let metrics = Arc::new(AppMetrics::new());
        let processor = JobProcessor::new(queue.clone(), Some(metrics.clone()));

        let content = b"Contact john@example.com for details.".to_vec();
        let job = Job::new(content, "text/plain");
        let job_id = queue.submit(job).await;

        // Mark as processing and process
        let pending_id = queue.get_pending().unwrap();
        assert_eq!(pending_id, job_id);
        processor.process_job(&job_id);

        let job = queue.get(&job_id).unwrap();
        assert_eq!(job.status, JobStatus::Completed);
        assert!(job.result.is_some());
        let result = job.result.unwrap();
        assert!(!result.entities.is_empty());
        assert!(result.extracted_text_length > 0);
        assert!(job.completed_at.is_some());

        assert_eq!(
            metrics
                .jobs_completed
                .load(std::sync::atomic::Ordering::Relaxed),
            1
        );
    }

    #[tokio::test]
    async fn test_process_unsupported_mime() {
        let queue = Arc::new(JobQueue::new());
        let metrics = Arc::new(AppMetrics::new());
        let processor = JobProcessor::new(queue.clone(), Some(metrics.clone()));

        let content = b"not a real video file".to_vec();
        let job = Job::new(content, "video/mp4");
        let job_id = queue.submit(job).await;

        let _ = queue.get_pending();
        processor.process_job(&job_id);

        let job = queue.get(&job_id).unwrap();
        assert_eq!(job.status, JobStatus::Failed);
        assert!(job.error.is_some());
        assert!(job.error.unwrap().contains("Unsupported"));
        assert_eq!(
            metrics
                .jobs_failed
                .load(std::sync::atomic::Ordering::Relaxed),
            1
        );
    }

    #[tokio::test]
    async fn test_get_pending_fifo() {
        let queue = Arc::new(JobQueue::new());
        let job1 = Job::new(b"first".to_vec(), "text/plain");
        let id1 = job1.id.clone();
        queue.submit(job1).await;

        // Small delay to ensure different created_at
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let job2 = Job::new(b"second".to_vec(), "text/plain");
        let _id2 = job2.id.clone();
        queue.submit(job2).await;

        // First pending should be the oldest
        let pending = queue.get_pending().unwrap();
        assert_eq!(pending, id1);
    }
}
