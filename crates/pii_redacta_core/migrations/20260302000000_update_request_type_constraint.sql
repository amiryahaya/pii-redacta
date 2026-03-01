-- Sprint 13: Update request_type CHECK constraint
-- Adds 'playground_file' for file-based playground submissions
-- Adds 'file_upload' to fix latent bug (used by upload_authenticated)

-- P1 fix: Fail loudly if unknown request_type values exist rather than
-- silently relabeling them, which could corrupt analytics data.
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM usage_logs WHERE request_type NOT IN (
      'playground', 'playground_file',
      'api_detect', 'api_redact', 'api_detect_stream',
      'file_upload'
    )
  ) THEN
    RAISE EXCEPTION 'Unknown request_type values exist in usage_logs; review before applying constraint';
  END IF;
END $$;

ALTER TABLE usage_logs DROP CONSTRAINT IF EXISTS usage_logs_request_type_check;
ALTER TABLE usage_logs ADD CONSTRAINT usage_logs_request_type_check
    CHECK (request_type IN (
        'playground', 'playground_file',
        'api_detect', 'api_redact', 'api_detect_stream',
        'file_upload'
    ));
