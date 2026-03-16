-- Migration 002: Users & Authentication

CREATE TABLE users (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email               TEXT UNIQUE NOT NULL,
    display_name        TEXT NOT NULL,
    avatar_url          TEXT,
    locale              TEXT NOT NULL DEFAULT 'en',
    is_premium          BOOLEAN NOT NULL DEFAULT FALSE,
    premium_until       TIMESTAMPTZ,
    cloud_used_bytes    BIGINT NOT NULL DEFAULT 0,
    cloud_limit_bytes   BIGINT NOT NULL DEFAULT 0,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ
);

CREATE INDEX idx_users_email ON users (email);

CREATE TABLE boost_tokens (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    camera_id       UUID NOT NULL REFERENCES cameras(id) ON DELETE CASCADE,
    days_added      SMALLINT NOT NULL DEFAULT 3 CHECK (days_added > 0),
    expires_at      TIMESTAMPTZ NOT NULL,
    applied_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_boost_tokens_camera_id ON boost_tokens (camera_id);
CREATE INDEX idx_boost_tokens_user_id ON boost_tokens (user_id);
CREATE INDEX idx_boost_tokens_expires_at ON boost_tokens (expires_at);

CREATE TABLE sessions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token           TEXT UNIQUE NOT NULL,
    expires_at      TIMESTAMPTZ NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sessions_token ON sessions (token);
CREATE INDEX idx_sessions_user_id ON sessions (user_id);
