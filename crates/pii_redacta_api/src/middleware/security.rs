//! Security middleware
//!
//! Provides security headers and request limits

use axum::{
    body::Body,
    http::{HeaderValue, Request, Response, StatusCode},
    middleware::Next,
};

/// Default maximum request body size (10MB)
pub const DEFAULT_MAX_BODY_SIZE: usize = 10 * 1024 * 1024;

/// Content Security Policy header value
const CSP_HEADER: &str = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self'; media-src 'self'; object-src 'none'; frame-ancestors 'none'; base-uri 'self'; form-action 'self';";

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

    // XSS protection (legacy, but still useful)
    headers.insert(
        "x-xss-protection",
        HeaderValue::from_static("1; mode=block"),
    );

    // Referrer policy
    headers.insert(
        "referrer-policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Content Security Policy
    headers.insert(
        "content-security-policy",
        HeaderValue::from_static(CSP_HEADER),
    );

    // Permissions Policy (formerly Feature Policy)
    headers.insert(
        "permissions-policy",
        HeaderValue::from_static("accelerometer=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(), usb=()"),
    );

    // HSTS: Enforce HTTPS (1 year, include subdomains) (S9-R2-18)
    headers.insert(
        "strict-transport-security",
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );

    response
}

/// Check request body size (fallback for Content-Length check)
/// Note: This is a simple check. For production, use tower_http::limit::RequestBodyLimitLayer
pub async fn limit_body_size(request: Request<Body>, next: Next) -> Response<Body> {
    // Check Content-Length header first (fast path)
    if let Some(content_length) = request.headers().get("content-length") {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<usize>() {
                if length > DEFAULT_MAX_BODY_SIZE {
                    return Response::builder()
                        .status(StatusCode::PAYLOAD_TOO_LARGE)
                        .body(Body::from(format!(
                            "Request body too large. Maximum size is {} bytes",
                            DEFAULT_MAX_BODY_SIZE
                        )))
                        .unwrap();
                }
            }
        }
    }

    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_default_max_body_size() {
        assert_eq!(DEFAULT_MAX_BODY_SIZE, 10 * 1024 * 1024);
    }

    #[test]
    fn test_csp_header_present() {
        assert!(CSP_HEADER.contains("default-src 'self'"));
        assert!(CSP_HEADER.contains("frame-ancestors 'none'"));
    }
}
