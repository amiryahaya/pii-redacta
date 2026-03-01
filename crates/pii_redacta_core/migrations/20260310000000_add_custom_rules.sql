-- Sprint 14: Custom Rules table
CREATE TABLE custom_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    description VARCHAR(500),
    pattern VARCHAR(1000) NOT NULL,
    entity_label VARCHAR(50) NOT NULL,
    confidence REAL NOT NULL DEFAULT 0.9,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT custom_rules_confidence_range CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT custom_rules_name_length CHECK (char_length(name) >= 1),
    CONSTRAINT custom_rules_pattern_length CHECK (char_length(pattern) >= 1)
);
CREATE INDEX idx_custom_rules_user_id ON custom_rules(user_id);
