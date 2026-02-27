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

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

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

/// Processing job
#[derive(Debug, Clone)]
pub struct Job {
    pub id: String,
    pub content: Vec<u8>,
    pub mime_type: String,
    pub status: JobStatus,
}

impl Job {
    pub fn new(content: Vec<u8>, mime_type: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content,
            mime_type: mime_type.to_string(),
            status: JobStatus::Pending,
        }
    }
}

/// In-memory job queue
pub struct JobQueue {
    jobs: Mutex<HashMap<String, Job>>,
}

impl JobQueue {
    pub fn new() -> Self {
        Self {
            jobs: Mutex::new(HashMap::new()),
        }
    }

    pub async fn submit(&self, job: Job) -> String {
        let id = job.id.clone();
        let mut jobs = self.jobs.lock().unwrap();
        jobs.insert(id.clone(), job);
        id
    }

    pub fn get(&self, job_id: &str) -> Option<Job> {
        let jobs = self.jobs.lock().unwrap();
        jobs.get(job_id).cloned()
    }
}

impl Default for JobQueue {
    fn default() -> Self {
        Self::new()
    }
}
