//! PII Redacta Core Library
//!
//! Core functionality for PII detection and tokenization.

pub mod detection;
pub mod error;
pub mod extraction;
pub mod tokenization;
pub mod types;

/// Core version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_available() {
        // Verify version constant is available and has expected format
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
        // Version should be in semver format (starts with digit)
        assert!(VERSION.chars().next().unwrap().is_ascii_digit());
    }
}
