//! Security middleware
//!
//! Provides security headers and request limits

use axum::{
    body::Body,
    http::{HeaderValue, Request, Response, StatusCode},
    middleware::Next,
};

/// Maximum request body size (10MB)
const MAX_BODY_SIZE: usize = 10 * 1024 * 1024;

/// Add security headers to responses
pub async fn security_headers(mut response: Response<Body>) -> Response<Body> {
    let headers = response.headers_mut();

    // Prevent MIME type sniffing
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );

    // Prevent clickjacking
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));

    // XSS protection
    headers.insert(
        "x-xss-protection",
        HeaderValue::from_static("1; mode=block"),
    );

    // Referrer policy
    headers.insert(
        "referrer-policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    response
}

/// Check request body size
pub async fn limit_body_size(request: Request<Body>, next: Next) -> Response<Body> {
    // Check Content-Length header
    if let Some(content_length) = request.headers().get("content-length") {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<usize>() {
                if length > MAX_BODY_SIZE {
                    return Response::builder()
                        .status(StatusCode::PAYLOAD_TOO_LARGE)
                        .body(Body::from("Request body too large"))
                        .unwrap();
                }
            }
        }
    }

    next.run(request).await
}
