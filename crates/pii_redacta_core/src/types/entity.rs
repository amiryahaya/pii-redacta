//! Entity types for detected PII

use serde::{Deserialize, Serialize};

/// Types of PII entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntityType {
    Email,
    PhoneNumber,
    MalaysianNric,
    PassportNumber,
    CreditCard,
    BankAccount,
    Address,
    PersonName,
    DateOfBirth,
    IpAddress,
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            EntityType::Email => "EMAIL",
            EntityType::PhoneNumber => "PHONE",
            EntityType::MalaysianNric => "MY_NRIC",
            EntityType::PassportNumber => "PASSPORT",
            EntityType::CreditCard => "CREDIT_CARD",
            EntityType::BankAccount => "BANK_ACCOUNT",
            EntityType::Address => "ADDRESS",
            EntityType::PersonName => "PERSON_NAME",
            EntityType::DateOfBirth => "DOB",
            EntityType::IpAddress => "IP_ADDRESS",
        };
        write!(f, "{}", s)
    }
}

/// Detected PII entity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub entity_type: EntityType,
    pub value: String,
    pub start: usize,
    pub end: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
}

impl Entity {
    /// Create a new entity
    pub fn new(entity_type: EntityType, value: &str, start: usize, end: usize) -> Self {
        Self {
            entity_type,
            value: value.to_string(),
            start,
            end,
            confidence: None,
        }
    }

    /// Set confidence score
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = Some(confidence);
        self
    }
}
