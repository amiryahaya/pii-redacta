-- Sprint 13: Update request_type CHECK constraint
-- Adds 'playground_file' for file-based playground submissions
-- Adds 'file_upload' to fix latent bug (used by upload_authenticated)

-- M1: Clean up any orphan rows that would violate the new constraint
UPDATE usage_logs SET request_type = 'playground'
WHERE request_type NOT IN (
    'playground', 'playground_file',
    'api_detect', 'api_redact', 'api_detect_stream',
    'file_upload'
);

ALTER TABLE usage_logs DROP CONSTRAINT IF EXISTS usage_logs_request_type_check;
ALTER TABLE usage_logs ADD CONSTRAINT usage_logs_request_type_check
    CHECK (request_type IN (
        'playground', 'playground_file',
        'api_detect', 'api_redact', 'api_detect_stream',
        'file_upload'
    ));
