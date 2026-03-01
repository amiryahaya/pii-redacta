-- Sprint 14: Batch Jobs and Items tables
CREATE TABLE batch_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'processing', 'completed', 'failed', 'partial')),
    total_items INT NOT NULL,
    completed_items INT NOT NULL DEFAULT 0,
    failed_items INT NOT NULL DEFAULT 0,
    redact BOOLEAN NOT NULL DEFAULT false,
    use_custom_rules BOOLEAN NOT NULL DEFAULT false,
    error_message VARCHAR(500),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_batch_jobs_user_id ON batch_jobs(user_id);

CREATE TABLE batch_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    batch_id UUID NOT NULL REFERENCES batch_jobs(id) ON DELETE CASCADE,
    item_index INT NOT NULL,
    input_text TEXT NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'processing', 'completed', 'failed')),
    entities JSONB,
    redacted_text TEXT,
    processing_time_ms INT,
    error_message VARCHAR(500),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);
CREATE INDEX idx_batch_items_batch_id ON batch_items(batch_id);
CREATE UNIQUE INDEX idx_batch_items_batch_index ON batch_items(batch_id, item_index);
