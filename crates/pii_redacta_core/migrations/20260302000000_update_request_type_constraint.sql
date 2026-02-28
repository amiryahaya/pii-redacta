-- Sprint 13: Update request_type CHECK constraint
-- Adds 'playground_file' for file-based playground submissions
-- Adds 'file_upload' to fix latent bug (used by upload_authenticated)
ALTER TABLE usage_logs DROP CONSTRAINT IF EXISTS usage_logs_request_type_check;
ALTER TABLE usage_logs ADD CONSTRAINT usage_logs_request_type_check
    CHECK (request_type IN (
        'playground', 'playground_file',
        'api_detect', 'api_redact', 'api_detect_stream',
        'file_upload'
    ));
