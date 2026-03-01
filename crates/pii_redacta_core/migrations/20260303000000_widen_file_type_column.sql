-- Sprint 13 fix: Widen file_type column to accommodate full MIME types
-- The DOCX MIME type (application/vnd.openxmlformats-officedocument.wordprocessingml.document)
-- is 71 characters, which exceeds the original VARCHAR(50).

ALTER TABLE usage_logs ALTER COLUMN file_type TYPE VARCHAR(255);
