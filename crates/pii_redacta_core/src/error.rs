//! Error types for PII Redacta Core

use thiserror::Error;

/// Main error type for the core library
#[derive(Error, Debug)]
pub enum PiiError {
    /// Invalid input error
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Extraction error
    #[error("Extraction failed: {0}")]
    ExtractionFailed(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, PiiError>;
