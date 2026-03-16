-- Placebo: Recording segments, clip requests, ratings

CREATE TABLE IF NOT EXISTS recording_segments (
    id                  TEXT PRIMARY KEY,
    camera_id           TEXT NOT NULL REFERENCES cameras(id) ON DELETE CASCADE,
    start_time          TEXT NOT NULL,
    end_time            TEXT NOT NULL,
    duration_seconds    INTEGER NOT NULL,
    storage_tier        TEXT NOT NULL DEFAULT 'hot' CHECK (storage_tier IN ('hot', 'warm', 'cold', 'archive')),
    codec               TEXT NOT NULL DEFAULT 'h264',
    file_path           TEXT NOT NULL,
    file_size_bytes     INTEGER NOT NULL DEFAULT 0,
    resolution          TEXT,
    created_at          TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_recording_segments_camera_id ON recording_segments(camera_id);
CREATE INDEX IF NOT EXISTS idx_recording_segments_time ON recording_segments(camera_id, start_time, end_time);

CREATE TABLE IF NOT EXISTS clip_requests (
    id                  TEXT PRIMARY KEY,
    camera_id           TEXT NOT NULL REFERENCES cameras(id) ON DELETE CASCADE,
    user_id             TEXT NOT NULL,
    start_time          TEXT NOT NULL,
    end_time            TEXT NOT NULL,
    status              TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'ready', 'failed')),
    output_url          TEXT,
    created_at          TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at        TEXT
);

CREATE INDEX IF NOT EXISTS idx_clip_requests_camera_id ON clip_requests(camera_id);
CREATE INDEX IF NOT EXISTS idx_clip_requests_user_id ON clip_requests(user_id);
CREATE INDEX IF NOT EXISTS idx_clip_requests_status ON clip_requests(status);

CREATE TABLE IF NOT EXISTS ratings (
    id                  TEXT PRIMARY KEY,
    camera_id           TEXT NOT NULL REFERENCES cameras(id) ON DELETE CASCADE,
    user_id             TEXT NOT NULL,
    score               INTEGER NOT NULL CHECK (score >= 1 AND score <= 5),
    created_at          TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(camera_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_ratings_camera_id ON ratings(camera_id);
