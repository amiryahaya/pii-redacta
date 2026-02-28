-- Add password_changed_at column to track when password was last changed.
-- Used to invalidate JWT tokens issued before a password change.
ALTER TABLE users ADD COLUMN password_changed_at TIMESTAMPTZ;
