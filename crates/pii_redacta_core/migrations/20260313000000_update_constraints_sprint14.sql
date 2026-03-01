-- Sprint 14: Update request_type constraint and tier limits

-- Safety check: ensure no unknown request_type values exist before altering constraint
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM usage_logs WHERE request_type NOT IN (
      'playground', 'playground_file',
      'api_detect', 'api_redact', 'api_detect_stream',
      'file_upload', 'batch_detect', 'batch_redact'
    )
  ) THEN
    RAISE EXCEPTION 'Unknown request_type values; review before applying';
  END IF;
END $$;

ALTER TABLE usage_logs DROP CONSTRAINT IF EXISTS usage_logs_request_type_check;
ALTER TABLE usage_logs ADD CONSTRAINT usage_logs_request_type_check
    CHECK (request_type IN (
        'playground', 'playground_file',
        'api_detect', 'api_redact', 'api_detect_stream',
        'file_upload', 'batch_detect', 'batch_redact'
    ));

-- Update Pro tier limits with batch/webhook/rule limits
UPDATE tiers SET limits = limits ||
    '{"maxBatchItems": 100, "maxWebhookEndpoints": 5, "maxCustomRules": 10}'::jsonb
WHERE name = 'pro';

UPDATE tiers SET limits = limits ||
    '{"maxBatchItems": 1000, "maxWebhookEndpoints": 20, "maxCustomRules": 50}'::jsonb
WHERE name = 'enterprise';
