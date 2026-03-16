-- Migration 004: Recordings, Clips, Ratings, Translations

CREATE TYPE storage_tier AS ENUM ('hot', 'warm', 'cold', 'archive');
CREATE TYPE clip_status AS ENUM ('pending', 'processing', 'ready', 'failed', 'expired');

CREATE TABLE recording_segments (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    camera_id           UUID NOT NULL REFERENCES cameras(id) ON DELETE CASCADE,
    start_time          TIMESTAMPTZ NOT NULL,
    end_time            TIMESTAMPTZ NOT NULL,
    duration_secs       INTEGER NOT NULL,
    storage_tier        storage_tier NOT NULL DEFAULT 'hot',
    codec               video_codec NOT NULL DEFAULT 'h264',
    file_path           TEXT NOT NULL,
    file_size_bytes     BIGINT NOT NULL DEFAULT 0,
    resolution          TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_recording_segments_camera_time ON recording_segments (camera_id, start_time, end_time);
CREATE INDEX idx_recording_segments_storage_tier ON recording_segments (storage_tier);

CREATE TABLE clip_requests (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    camera_id       UUID NOT NULL REFERENCES cameras(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    start_time      TIMESTAMPTZ NOT NULL,
    end_time        TIMESTAMPTZ NOT NULL,
    status          clip_status NOT NULL DEFAULT 'pending',
    output_url      TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at    TIMESTAMPTZ
);

CREATE INDEX idx_clip_requests_user_id ON clip_requests (user_id);
CREATE INDEX idx_clip_requests_camera_id ON clip_requests (camera_id);

CREATE TABLE ratings (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    camera_id       UUID NOT NULL REFERENCES cameras(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    score           SMALLINT NOT NULL CHECK (score >= 1 AND score <= 5),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (camera_id, user_id)
);

CREATE INDEX idx_ratings_camera_id ON ratings (camera_id);

CREATE TABLE camera_translations (
    camera_id       UUID NOT NULL REFERENCES cameras(id) ON DELETE CASCADE,
    lang            TEXT NOT NULL,
    description     TEXT NOT NULL,
    translated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (camera_id, lang)
);
