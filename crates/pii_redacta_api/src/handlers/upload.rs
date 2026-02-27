//! File upload handler

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::{extract::State, Json};
use serde::Serialize;
use std::sync::Arc;

use super::{Job, JobQueue};

#[derive(Serialize)]
pub struct UploadResponse {
    pub job_id: String,
    pub status: String,
}

pub async fn upload(
    State(queue): State<Arc<JobQueue>>,
    request: Request<Body>,
) -> (StatusCode, Json<UploadResponse>) {
    // Get content type first
    let content_type = request
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    if !content_type.starts_with("multipart/form-data") {
        return (
            StatusCode::BAD_REQUEST,
            Json(UploadResponse {
                job_id: "".to_string(),
                status: "invalid content type".to_string(),
            }),
        );
    }

    // Extract boundary
    let boundary = content_type
        .split("boundary=")
        .nth(1)
        .unwrap_or("----WebKitFormBoundary")
        .to_string();

    // Collect body bytes
    let body_bytes = match axum::body::to_bytes(request.into_body(), usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(UploadResponse {
                    job_id: "".to_string(),
                    status: "failed to read body".to_string(),
                }),
            );
        }
    };

    // Parse multipart content (simplified)
    let body_str = String::from_utf8_lossy(&body_bytes);
    let boundary_str = format!("--{}", boundary);

    // Find the file content
    let mut found_file = false;
    let mut file_content = Vec::new();
    let mut mime_type = "application/octet-stream".to_string();

    for part in body_str.split(&boundary_str) {
        if part.contains("Content-Disposition: form-data") && part.contains("name=\"file\"") {
            found_file = true;

            // Extract filename and content-type
            if let Some(ct_line) = part.lines().find(|l| l.starts_with("Content-Type:")) {
                mime_type = ct_line
                    .split(':')
                    .nth(1)
                    .unwrap_or("application/octet-stream")
                    .trim()
                    .to_string();
            }

            // Find the empty line that separates headers from content
            if let Some(content_start) = part.find("\r\n\r\n") {
                let content = &part[content_start + 4..];
                // Remove trailing \r\n before the boundary
                let content = content.trim_end_matches("\r\n");
                file_content = content.as_bytes().to_vec();
            }
            break;
        }
    }

    if !found_file || file_content.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(UploadResponse {
                job_id: "".to_string(),
                status: "no file provided".to_string(),
            }),
        );
    }

    // Create and submit job
    let job = Job::new(file_content, &mime_type);
    let job_id = queue.submit(job).await;

    (
        StatusCode::ACCEPTED,
        Json(UploadResponse {
            job_id,
            status: "accepted".to_string(),
        }),
    )
}

#[cfg(test)]
#[path = "upload_test.rs"]
mod tests;
