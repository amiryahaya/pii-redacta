//! Pattern-based PII detection
//!
//! Uses regex patterns for fast PII identification.

use crate::types::{Entity, EntityType};
use once_cell::sync::Lazy;
use regex::Regex;

// Regex patterns compiled once for performance
static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").expect("Valid regex")
});

// Malaysian NRIC: YYMMDD-BB-NNNN or YYMMDDBBNNNN
static NRIC_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b\d{6}-?\d{2}-?\d{4}\b").expect("Valid regex"));

// Malaysian phone numbers: +60, 01x, etc.
// Matches: 012-3456789, 0123456789, +60 12-345 6789, +60123456789
static PHONE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:\+?60[\s-]?\d{1,2}[\s-]?\d{3}[\s-]?\d{4,5}|01\d[\s-]?\d{3}[\s-]?\d{4,5})")
        .expect("Valid regex")
});

// Credit card: 4 groups of 4 digits, separated by spaces, dashes, or nothing
static CREDIT_CARD_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b(?:\d{4}[\s-]?){3}\d{4}\b").expect("Valid regex"));

/// Pattern-based PII detector
pub struct PatternDetector;

impl PatternDetector {
    /// Create a new pattern detector
    pub fn new() -> Self {
        Self
    }

    /// Detect email addresses
    pub fn detect_email(&self, text: &str) -> Vec<Entity> {
        self.find_entities(text, &EMAIL_REGEX, EntityType::Email)
    }

    /// Detect Malaysian NRIC numbers
    pub fn detect_nric(&self, text: &str) -> Vec<Entity> {
        self.find_entities(text, &NRIC_REGEX, EntityType::MalaysianNric)
    }

    /// Detect phone numbers
    pub fn detect_phone(&self, text: &str) -> Vec<Entity> {
        self.find_entities(text, &PHONE_REGEX, EntityType::PhoneNumber)
    }

    /// Detect credit card numbers
    pub fn detect_credit_card(&self, text: &str) -> Vec<Entity> {
        self.find_entities(text, &CREDIT_CARD_REGEX, EntityType::CreditCard)
    }

    /// Detect all PII types
    pub fn detect_all(&self, text: &str) -> Vec<Entity> {
        let mut all = Vec::new();
        all.extend(self.detect_email(text));
        all.extend(self.detect_nric(text));
        all.extend(self.detect_phone(text));
        all.extend(self.detect_credit_card(text));

        // Sort by position in text
        all.sort_by_key(|e| e.start);
        all
    }

    /// Helper to find entities using a regex
    fn find_entities(&self, text: &str, regex: &Regex, entity_type: EntityType) -> Vec<Entity> {
        regex
            .find_iter(text)
            .map(|m| Entity::new(entity_type, m.as_str(), m.start(), m.end()).with_confidence(0.95))
            .collect()
    }
}

impl Default for PatternDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "patterns_test.rs"]
mod tests;
