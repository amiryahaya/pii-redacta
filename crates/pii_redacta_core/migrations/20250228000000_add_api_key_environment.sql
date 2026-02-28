-- Add environment column to api_keys table
-- Stores whether the key is for 'live' or 'test' environment
ALTER TABLE api_keys ADD COLUMN IF NOT EXISTS environment VARCHAR(10) NOT NULL DEFAULT 'live';
