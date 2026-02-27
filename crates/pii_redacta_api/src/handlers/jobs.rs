//! Job status handler

use axum::extract::Path;
use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use std::sync::Arc;

use super::JobQueue;

#[derive(Serialize)]
pub struct JobStatusResponse {
    pub job_id: String,
    pub status: String,
}

pub async fn get_job_status(
    State(queue): State<Arc<JobQueue>>,
    Path(job_id): Path<String>,
) -> (StatusCode, Json<JobStatusResponse>) {
    match queue.get(&job_id) {
        Some(job) => (
            StatusCode::OK,
            Json(JobStatusResponse {
                job_id: job.id,
                status: job.status.to_string(),
            }),
        ),
        None => (
            StatusCode::NOT_FOUND,
            Json(JobStatusResponse {
                job_id,
                status: "not found".to_string(),
            }),
        ),
    }
}

#[cfg(test)]
#[path = "jobs_test.rs"]
mod tests;
