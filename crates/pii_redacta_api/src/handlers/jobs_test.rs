//! Job status handler tests
//!
//! Sprint 6: File Upload API & Async Processing

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use std::sync::Arc;

use super::get_job_status;
use crate::handlers::{Job, JobQueue};

fn create_test_state() -> Arc<JobQueue> {
    Arc::new(JobQueue::new())
}

#[tokio::test]
async fn test_get_job_status_found() {
    let state = create_test_state();

    // First create a job
    let job_id = state.submit(Job::new(vec![1, 2, 3], "text/plain")).await;

    let (status, Json(response)) = get_job_status(State(state), Path(job_id)).await;

    assert_eq!(status, StatusCode::OK);
    assert!(!response.job_id.is_empty());
}

#[tokio::test]
async fn test_get_job_status_not_found() {
    let state = create_test_state();

    let (status, _) = get_job_status(State(state), Path("non-existent-job-id".to_string())).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_job_status_progression() {
    let state = create_test_state();

    // Submit a job
    let job_id = state.submit(Job::new(vec![1, 2, 3], "text/plain")).await;

    // Check initial status
    let (_, Json(response)) = get_job_status(State(state.clone()), Path(job_id.clone())).await;
    assert!(
        response.status == "pending"
            || response.status == "processing"
            || response.status == "completed"
    );

    // Job ID should match
    assert_eq!(response.job_id, job_id);
}

#[tokio::test]
async fn test_job_queue_multiple_jobs() {
    let state = create_test_state();

    // Submit multiple jobs
    let job_id1 = state.submit(Job::new(vec![1], "text/plain")).await;
    let job_id2 = state.submit(Job::new(vec![2], "text/plain")).await;
    let job_id3 = state.submit(Job::new(vec![3], "text/plain")).await;

    // All should be retrievable
    let (status1, _) = get_job_status(State(state.clone()), Path(job_id1)).await;
    let (status2, _) = get_job_status(State(state.clone()), Path(job_id2)).await;
    let (status3, _) = get_job_status(State(state), Path(job_id3)).await;

    assert_eq!(status1, StatusCode::OK);
    assert_eq!(status2, StatusCode::OK);
    assert_eq!(status3, StatusCode::OK);
}
