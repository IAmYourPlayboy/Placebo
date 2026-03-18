-- 007_auth_password.sql
-- Add authentication fields to users table

ALTER TABLE users
    ADD COLUMN password_hash     TEXT,
    ADD COLUMN email_verified    BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN email_verify_token TEXT,
    ADD COLUMN password_reset_token TEXT,
    ADD COLUMN password_reset_expires TIMESTAMPTZ;

-- Index for email verification token lookup
CREATE INDEX idx_users_email_verify_token ON users (email_verify_token) WHERE email_verify_token IS NOT NULL;

-- Index for password reset token lookup
CREATE INDEX idx_users_password_reset_token ON users (password_reset_token) WHERE password_reset_token IS NOT NULL;

-- Mark existing dev users as email-verified (they don't have passwords – dev tokens only)
UPDATE users SET email_verified = TRUE WHERE email LIKE '%@placebo.dev';
