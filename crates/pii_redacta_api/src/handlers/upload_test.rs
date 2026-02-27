//! File upload handler tests
//!
//! Sprint 6: File Upload API & Async Processing

use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::Json;
use std::sync::Arc;

use super::upload;
use crate::handlers::JobQueue;

fn create_test_state() -> Arc<JobQueue> {
    Arc::new(JobQueue::new())
}

#[tokio::test]
async fn test_upload_txt_file() {
    let state = create_test_state();

    // Create multipart body for a simple text file
    let boundary = "----WebKitFormBoundary";
    let body = "------WebKitFormBoundary\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n\
         Content-Type: text/plain\r\n\r\n\
         Email: test@example.com\r\n\
         ------WebKitFormBoundary--\r\n"
        .to_string();

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/upload")
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(body))
        .unwrap();

    let (status, Json(response)) = upload(State(state), request).await;

    assert_eq!(status, StatusCode::ACCEPTED);
    assert!(!response.job_id.is_empty());
}

#[tokio::test]
async fn test_upload_returns_job_id() {
    let state = create_test_state();

    let boundary = "----WebKitFormBoundary";
    let body = "------WebKitFormBoundary\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n\
         Content-Type: text/plain\r\n\r\n\
         Test content\r\n\
         ------WebKitFormBoundary--\r\n"
        .to_string();

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/upload")
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(body))
        .unwrap();

    let (_, Json(response)) = upload(State(state), request).await;

    // Job ID should be a valid UUID-like format
    assert!(response.job_id.len() >= 20);
    assert!(response.status == "accepted" || response.status == "processing");
}

#[tokio::test]
async fn test_upload_pdf_file() {
    let state = create_test_state();

    let boundary = "----WebKitFormBoundary";
    let body = "------WebKitFormBoundary\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"test.pdf\"\r\n\
         Content-Type: application/pdf\r\n\r\n\
         %PDF-1.4 test content\r\n\
         ------WebKitFormBoundary--\r\n"
        .to_string();

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/upload")
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(body))
        .unwrap();

    let (status, _) = upload(State(state), request).await;

    assert_eq!(status, StatusCode::ACCEPTED);
}

#[tokio::test]
async fn test_upload_missing_file_bad_request() {
    let state = create_test_state();

    // Request without file field
    let boundary = "----WebKitFormBoundary";
    let body = "------WebKitFormBoundary\r\n\
         Content-Disposition: form-data; name=\"other\"\r\n\r\n\
         value\r\n\
         ------WebKitFormBoundary--\r\n"
        .to_string();

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/upload")
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(body))
        .unwrap();

    let (status, _) = upload(State(state), request).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_upload_unsupported_type() {
    let state = create_test_state();

    let boundary = "----WebKitFormBoundary";
    let body = "------WebKitFormBoundary\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"test.xyz\"\r\n\
         Content-Type: application/unknown\r\n\r\n\
         binary content\r\n\
         ------WebKitFormBoundary--\r\n"
        .to_string();

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/upload")
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(body))
        .unwrap();

    let (status, _) = upload(State(state), request).await;

    // Should accept but may fail processing later
    assert!(status == StatusCode::ACCEPTED || status == StatusCode::BAD_REQUEST);
}
