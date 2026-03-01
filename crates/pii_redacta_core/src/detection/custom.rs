//! Custom rule detection engine
//!
//! Sprint 14: Allows users to define custom regex patterns for PII detection.

use crate::types::{Entity, EntityType};
use regex::RegexBuilder;
use uuid::Uuid;

/// Maximum pattern length to prevent ReDoS attacks
const MAX_PATTERN_LENGTH: usize = 500;

/// Maximum compiled regex size (1 MB)
const MAX_REGEX_SIZE: usize = 1 << 20;

/// A user-defined detection rule
#[derive(Debug, Clone)]
pub struct CustomRule {
    pub id: Uuid,
    pub name: String,
    pub pattern: String,
    pub entity_label: String,
    pub confidence: f32,
}

/// Detector for user-defined custom rules
pub struct CustomRuleDetector;

impl CustomRuleDetector {
    pub fn new() -> Self {
        Self
    }

    /// Validate a regex pattern. Returns `Ok(())` if valid, `Err(message)` if invalid.
    pub fn validate_pattern(pattern: &str) -> Result<(), String> {
        if pattern.is_empty() {
            return Err("Pattern cannot be empty".to_string());
        }
        if pattern.len() > MAX_PATTERN_LENGTH {
            return Err(format!(
                "Pattern too long ({} chars, max {})",
                pattern.len(),
                MAX_PATTERN_LENGTH
            ));
        }
        RegexBuilder::new(pattern)
            .size_limit(MAX_REGEX_SIZE)
            .build()
            .map(|_| ())
            .map_err(|e| format!("Invalid regex: {e}"))
    }

    /// Run custom rules against text, returning detected entities.
    pub fn detect(&self, text: &str, rules: &[CustomRule]) -> Vec<Entity> {
        let mut entities = Vec::new();

        for rule in rules {
            let regex = match RegexBuilder::new(&rule.pattern)
                .size_limit(MAX_REGEX_SIZE)
                .build()
            {
                Ok(r) => r,
                Err(_) => {
                    // Skip rules with invalid patterns (already validated at creation time)
                    continue;
                }
            };

            for mat in regex.find_iter(text) {
                entities.push(Entity {
                    entity_type: EntityType::Custom,
                    value: mat.as_str().to_string(),
                    start: mat.start(),
                    end: mat.end(),
                    confidence: Some(rule.confidence),
                    custom_label: Some(rule.entity_label.clone()),
                });
            }
        }

        entities
    }
}

impl Default for CustomRuleDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "custom_test.rs"]
mod tests;
