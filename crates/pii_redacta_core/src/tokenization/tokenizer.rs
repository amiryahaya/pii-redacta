//! Tokenizer implementation
//!
//! Sprint 3: Tokenization Engine

use crate::types::{Entity, EntityType};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Token generator for creating deterministic, tenant-scoped tokens
pub struct TokenGenerator {
    tenant_id: String,
}

impl TokenGenerator {
    /// Create a new token generator for a tenant
    pub fn new(tenant_id: &str) -> Self {
        Self {
            tenant_id: tenant_id.to_string(),
        }
    }

    /// Generate a deterministic token for an entity value
    ///
    /// The token is a hash of (tenant_id + value) ensuring:
    /// - Same tenant + value = same token (deterministic)
    /// - Different tenants = different tokens (isolation)
    /// - Original value cannot be reversed from token
    pub fn generate(&self, entity_type: EntityType, value: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&self.tenant_id);
        hasher.update(value.as_bytes());

        // Use first 12 hex chars of hash (48 bits) for readable but unique tokens
        let hash = hex::encode(&hasher.finalize()[..6]);

        format!("<<PII_{}_{}>>", entity_type, hash.to_uppercase())
    }
}

/// Tokenizer for redacting and restoring PII
pub struct Tokenizer {
    generator: TokenGenerator,
}

/// Type alias for token map
pub type TokenMap = HashMap<String, String>;

impl Tokenizer {
    /// Create a new tokenizer for a tenant
    pub fn new(tenant_id: &str) -> Self {
        Self {
            generator: TokenGenerator::new(tenant_id),
        }
    }

    /// Tokenize text by replacing entities with tokens
    ///
    /// Returns (tokenized_text, token_map) where token_map can be used
    /// to restore the original values via detokenize()
    pub fn tokenize(&self, text: &str, entities: &[Entity]) -> (String, TokenMap) {
        if entities.is_empty() {
            return (text.to_string(), HashMap::new());
        }

        let mut token_map = HashMap::new();
        let mut result = text.to_string();

        // Sort entities by start position (descending) to replace from end
        // This prevents position shifts from affecting subsequent replacements
        let mut sorted_entities: Vec<_> = entities.to_vec();
        sorted_entities.sort_by_key(|e| std::cmp::Reverse(e.start));

        // Remove overlapping entities (keep larger ones)
        sorted_entities = self.remove_overlapping(sorted_entities);

        for entity in sorted_entities {
            let token = self.generator.generate(entity.entity_type, &entity.value);

            // Store mapping from token to original value
            token_map.insert(token.clone(), entity.value.clone());

            // Replace in result string
            let start = entity.start;
            let end = entity.end;
            if start < result.len() && end <= result.len() {
                result.replace_range(start..end, &token);
            }
        }

        (result, token_map)
    }

    /// Detokenize text by replacing tokens with original values
    pub fn detokenize(&self, tokenized: &str, token_map: &TokenMap) -> String {
        if token_map.is_empty() {
            return tokenized.to_string();
        }

        let mut result = tokenized.to_string();

        // Replace tokens in reverse order of length (longest first)
        // This prevents partial replacements of similar tokens
        let mut tokens: Vec<_> = token_map.iter().collect();
        tokens.sort_by_key(|(k, _)| std::cmp::Reverse(k.len()));

        for (token, original) in tokens {
            result = result.replace(token, original);
        }

        result
    }

    /// Remove overlapping entities, keeping the larger ones
    fn remove_overlapping(&self, entities: Vec<Entity>) -> Vec<Entity> {
        if entities.len() <= 1 {
            return entities;
        }

        let mut result = Vec::new();

        for entity in entities {
            let overlaps = result.iter().any(|e: &Entity| {
                // Check if current entity overlaps with any in result
                entity.start < e.end && entity.end > e.start
            });

            if !overlaps {
                result.push(entity);
            }
        }

        result
    }
}

#[cfg(test)]
#[path = "tokenizer_test.rs"]
mod tests;
