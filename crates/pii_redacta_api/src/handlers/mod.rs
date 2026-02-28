//! API handlers

pub mod api_keys;
pub mod auth;
pub mod detection;
pub mod health;
pub mod jobs;
pub mod metrics;
pub mod subscription;
pub mod upload;
pub mod usage;

use axum::{
    extract::{Extension, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

/// Admin stats response (S12-2c)
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminStatsResponse {
    pub total_users: i64,
    pub total_api_keys: i64,
}

/// Admin-only stats handler — returns basic system statistics.
/// Protected by admin middleware (S12-2c).
/// Extracts `AdminUser` to prove admin middleware ran — the value itself is unused
/// but its presence ensures the middleware inserted it.
pub async fn admin_stats(
    State(state): State<crate::AppState>,
    Extension(_admin): Extension<crate::extractors::AdminUser>,
) -> Result<Json<AdminStatsResponse>, axum::response::Response> {
    let total_users =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE deleted_at IS NULL")
            .fetch_one(state.db.pool())
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "admin_stats: failed to count users");
                admin_error_response()
            })?;

    let total_api_keys =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM api_keys WHERE is_active = true")
            .fetch_one(state.db.pool())
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "admin_stats: failed to count api_keys");
                admin_error_response()
            })?;

    Ok(Json(AdminStatsResponse {
        total_users,
        total_api_keys,
    }))
}

/// Build a consistent JSON error response for admin endpoints (L4).
fn admin_error_response() -> axum::response::Response {
    let body = serde_json::json!({
        "error": {
            "code": 500,
            "message": "An unexpected error occurred",
        }
    });
    (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
}

/// Job status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            JobStatus::Pending => "pending",
            JobStatus::Processing => "processing",
            JobStatus::Completed => "completed",
            JobStatus::Failed => "failed",
        };
        write!(f, "{}", s)
    }
}

/// Result of a completed job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    pub entities: Vec<pii_redacta_core::types::Entity>,
    pub processing_time_ms: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redacted_text: Option<String>,
    pub extracted_text_length: usize,
}

/// Processing job
#[derive(Debug, Clone)]
pub struct Job {
    pub id: String,
    pub content: Vec<u8>,
    pub mime_type: String,
    pub status: JobStatus,
    pub file_name: Option<String>,
    pub result: Option<JobResult>,
    pub error: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Job {
    pub fn new(content: Vec<u8>, mime_type: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content,
            mime_type: mime_type.to_string(),
            status: JobStatus::Pending,
            file_name: None,
            result: None,
            error: None,
            created_at: chrono::Utc::now(),
            completed_at: None,
        }
    }
}

/// In-memory job queue with automatic eviction of finished jobs.
///
/// Completed and failed jobs are retained for [`JOB_RETENTION`] so clients
/// can poll results, then evicted to bound memory usage.
pub struct JobQueue {
    jobs: Mutex<HashMap<String, Job>>,
}

/// How long finished (completed/failed) jobs are kept before eviction.
const JOB_RETENTION: chrono::Duration = chrono::Duration::minutes(30);

/// Eviction runs when the map exceeds this many entries.
const EVICTION_THRESHOLD: usize = 1_000;

impl JobQueue {
    pub fn new() -> Self {
        Self {
            jobs: Mutex::new(HashMap::new()),
        }
    }

    pub async fn submit(&self, job: Job) -> String {
        let id = job.id.clone();
        let mut jobs = self.jobs.lock().expect("JobQueue mutex poisoned");
        // Evict stale finished jobs when the map grows large
        Self::maybe_evict(&mut jobs);
        jobs.insert(id.clone(), job);
        id
    }

    pub fn get(&self, job_id: &str) -> Option<Job> {
        let jobs = self.jobs.lock().expect("JobQueue mutex poisoned");
        jobs.get(job_id).cloned()
    }

    /// Update a job's status, result, and error fields
    pub fn update_job(
        &self,
        job_id: &str,
        status: JobStatus,
        result: Option<JobResult>,
        error: Option<String>,
    ) {
        let mut jobs = self.jobs.lock().expect("JobQueue mutex poisoned");
        if let Some(job) = jobs.get_mut(job_id) {
            job.status = status;
            job.result = result;
            job.error = error;
            job.completed_at = Some(chrono::Utc::now());
            // Drop file content after processing to free memory
            job.content = Vec::new();
        }
    }

    /// Get the oldest pending job ID and mark it as Processing
    pub fn get_pending(&self) -> Option<String> {
        let mut jobs = self.jobs.lock().expect("JobQueue mutex poisoned");
        // Find the oldest pending job by created_at
        let oldest = jobs
            .values()
            .filter(|j| j.status == JobStatus::Pending)
            .min_by_key(|j| j.created_at)
            .map(|j| j.id.clone());

        if let Some(ref id) = oldest {
            if let Some(job) = jobs.get_mut(id) {
                job.status = JobStatus::Processing;
            }
        }
        oldest
    }

    /// Evict completed/failed jobs older than [`JOB_RETENTION`] when the
    /// map exceeds [`EVICTION_THRESHOLD`].
    fn maybe_evict(jobs: &mut HashMap<String, Job>) {
        if jobs.len() <= EVICTION_THRESHOLD {
            return;
        }
        let cutoff = chrono::Utc::now() - JOB_RETENTION;
        jobs.retain(|_, job| {
            // Keep pending/processing jobs unconditionally
            if job.status == JobStatus::Pending || job.status == JobStatus::Processing {
                return true;
            }
            // Keep finished jobs that are newer than the cutoff
            job.completed_at.map_or(true, |t| t > cutoff)
        });
    }
}

impl Default for JobQueue {
    fn default() -> Self {
        Self::new()
    }
}
