//! PII Redacta API Server
//!
//! REST API for PII detection and redaction.

use pii_redacta_core::VERSION;

#[tokio::main]
async fn main() {
    println!("PII Redacta API v{VERSION} - Sprint 1 Foundation");
    // Placeholder for Sprint 1 - will be implemented in Sprint 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_version_accessible() {
        // Verify we can access core library version
        assert!(!VERSION.is_empty());
    }
}
