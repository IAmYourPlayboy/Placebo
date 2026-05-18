-- 009_camera_stream_sources.sql
-- Adds a first-class stream source descriptor for cameras.
-- The existing stream_url/stream_type columns remain for legacy/RTSP ingest
-- but are NEVER exposed via the public API.

CREATE TYPE stream_source_type AS ENUM (
    'youtube_live',
    'direct_hls',
    'loop_mp4',
    'rtsp'
);

ALTER TABLE cameras
    ADD COLUMN stream_source_type   stream_source_type,
    ADD COLUMN stream_source_config JSONB NOT NULL DEFAULT '{}'::jsonb;

CREATE INDEX idx_cameras_stream_source_type ON cameras (stream_source_type);
