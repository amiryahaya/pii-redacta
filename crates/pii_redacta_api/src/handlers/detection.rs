//! Detection handler

use axum::{extract::State, http::StatusCode, Json};
use pii_redacta_core::detection::PatternDetector;
use pii_redacta_core::tokenization::Tokenizer;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
pub struct DetectRequest {
    pub text: String,
    pub options: Option<DetectionOptions>,
}

#[derive(Deserialize)]
pub struct DetectionOptions {
    pub redact: bool,
    pub tenant_id: Option<String>,
}

#[derive(Serialize)]
pub struct DetectResponse {
    pub entities: Vec<pii_redacta_core::types::Entity>,
    pub processing_time_ms: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redacted_text: Option<String>,
}

pub async fn detect(
    State(detector): State<Arc<PatternDetector>>,
    Json(request): Json<DetectRequest>,
) -> (StatusCode, Json<DetectResponse>) {
    // Validate input
    if request.text.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(DetectResponse {
                entities: Vec::new(),
                processing_time_ms: 0.0,
                redacted_text: None,
            }),
        );
    }

    let start = std::time::Instant::now();

    // Detect entities
    let entities = detector.detect_all(&request.text);
    let processing_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    // Handle redaction if requested
    let redacted_text = if let Some(options) = request.options {
        if options.redact && !entities.is_empty() {
            let tenant_id = options.tenant_id.as_deref().unwrap_or("default");
            let tokenizer = Tokenizer::new(tenant_id);
            let (tokenized, _token_map) = tokenizer.tokenize(&request.text, &entities);
            Some(tokenized)
        } else {
            None
        }
    } else {
        None
    };

    (
        StatusCode::OK,
        Json(DetectResponse {
            entities,
            processing_time_ms,
            redacted_text,
        }),
    )
}

#[cfg(test)]
#[path = "detection_test.rs"]
mod tests;
