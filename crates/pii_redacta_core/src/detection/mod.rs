//! Detection module for PII identification
//!
//! Provides pattern-based detection for common PII types.

pub mod custom;
pub mod patterns;

pub use custom::{CustomRule, CustomRuleDetector};
pub use patterns::PatternDetector;
