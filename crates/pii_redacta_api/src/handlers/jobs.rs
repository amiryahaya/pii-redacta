//! Job status handler

use axum::extract::Path;
use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use std::sync::Arc;

use super::{JobQueue, JobResult};
use crate::extractors::AuthUser;
use crate::AppState;
use axum::extract::Extension;

#[derive(Serialize)]
pub struct JobStatusResponse {
    pub job_id: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<JobResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
}

/// MVP job status handler (no auth, uses Arc<JobQueue> state)
pub async fn get_job_status(
    State(queue): State<Arc<JobQueue>>,
    Path(job_id): Path<String>,
) -> (StatusCode, Json<JobStatusResponse>) {
    build_job_response(&queue, &job_id)
}

/// Authenticated job status handler
pub async fn get_job_status_authenticated(
    State(state): State<AppState>,
    Extension(_auth_user): Extension<AuthUser>,
    Path(job_id): Path<String>,
) -> (StatusCode, Json<JobStatusResponse>) {
    build_job_response(&state.job_queue, &job_id)
}

fn build_job_response(queue: &JobQueue, job_id: &str) -> (StatusCode, Json<JobStatusResponse>) {
    match queue.get(job_id) {
        Some(job) => (
            StatusCode::OK,
            Json(JobStatusResponse {
                job_id: job.id,
                status: job.status.to_string(),
                result: job.result,
                error: job.error,
                created_at: Some(job.created_at.to_rfc3339()),
                completed_at: job.completed_at.map(|t| t.to_rfc3339()),
            }),
        ),
        None => (
            StatusCode::NOT_FOUND,
            Json(JobStatusResponse {
                job_id: job_id.to_string(),
                status: "not found".to_string(),
                result: None,
                error: None,
                created_at: None,
                completed_at: None,
            }),
        ),
    }
}

#[cfg(test)]
#[path = "jobs_test.rs"]
mod tests;
