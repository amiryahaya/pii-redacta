//! File upload handler

use axum::body::Body;
use axum::extract::Extension;
use axum::http::{Request, StatusCode};
use axum::{extract::State, Json};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;

use super::{Job, JobQueue};
use crate::extractors::AuthUser;
use crate::middleware::security::DEFAULT_MAX_BODY_SIZE;
use crate::AppState;
use axum::extract::ConnectInfo;

#[derive(Serialize)]
pub struct UploadResponse {
    pub job_id: String,
    pub status: String,
}

/// MVP upload handler (no auth)
pub async fn upload(
    State(queue): State<Arc<JobQueue>>,
    request: Request<Body>,
) -> (StatusCode, Json<UploadResponse>) {
    match parse_upload(request).await {
        Ok((content, mime_type)) => {
            let job = Job::new(content, &mime_type);
            let job_id = queue.submit(job).await;
            (
                StatusCode::ACCEPTED,
                Json(UploadResponse {
                    job_id,
                    status: "accepted".to_string(),
                }),
            )
        }
        Err(response) => response,
    }
}

/// Authenticated upload handler — records usage and uses AppState job queue
pub async fn upload_authenticated(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    request: Request<Body>,
) -> (StatusCode, Json<UploadResponse>) {
    match parse_upload(request).await {
        Ok((content, mime_type)) => {
            let file_size = content.len() as i32;
            let job = Job::new(content, &mime_type);
            let job_id = state.job_queue.submit(job).await;

            state.metrics.record_upload();

            // Fire-and-forget usage recording
            let pool = state.db.pool().clone();
            let user_id = auth_user.user_id;
            let ip = connect_info.map(|ci| ci.0.ip().to_string());
            let file_type = mime_type.clone();
            tokio::spawn(async move {
                let record = pii_redacta_core::db::usage::UsageRecord {
                    user_id,
                    api_key_id: None,
                    request_type: "file_upload",
                    file_name: None,
                    file_size_bytes: Some(file_size),
                    file_type: Some(&file_type),
                    processing_time_ms: None,
                    page_count: None,
                    detections_count: None,
                    success: true,
                    error_message: None,
                    ip_address: ip.as_deref(),
                };
                if let Err(e) = pii_redacta_core::db::usage::record_usage(&pool, &record).await {
                    tracing::warn!("Failed to record upload usage: {e}");
                }
            });

            (
                StatusCode::ACCEPTED,
                Json(UploadResponse {
                    job_id,
                    status: "accepted".to_string(),
                }),
            )
        }
        Err(response) => response,
    }
}

/// Parse the multipart upload request, returning file content and MIME type.
async fn parse_upload(
    request: Request<Body>,
) -> Result<(Vec<u8>, String), (StatusCode, Json<UploadResponse>)> {
    let content_type = request
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    if !content_type.starts_with("multipart/form-data") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(UploadResponse {
                job_id: "".to_string(),
                status: "invalid content type".to_string(),
            }),
        ));
    }

    // Extract boundary — require it to be present (S11-QA-08)
    let boundary = match content_type.split("boundary=").nth(1) {
        Some(b) => b.to_string(),
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(UploadResponse {
                    job_id: "".to_string(),
                    status: "missing multipart boundary".to_string(),
                }),
            ));
        }
    };

    // Collect body bytes — bounded to match middleware limit (S11-QA-07)
    let body_bytes = match axum::body::to_bytes(request.into_body(), DEFAULT_MAX_BODY_SIZE).await {
        Ok(bytes) => bytes,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(UploadResponse {
                    job_id: "".to_string(),
                    status: "failed to read body".to_string(),
                }),
            ));
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
        return Err((
            StatusCode::BAD_REQUEST,
            Json(UploadResponse {
                job_id: "".to_string(),
                status: "no file provided".to_string(),
            }),
        ));
    }

    Ok((file_content, mime_type))
}

#[cfg(test)]
#[path = "upload_test.rs"]
mod tests;
