-- Migration 001: Cameras
-- Requires: PostGIS extension, pg_trgm extension

CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- ENUM types
CREATE TYPE camera_type AS ENUM ('public', 'enterprise', 'yourself');
CREATE TYPE retention_tier AS ENUM ('tier1', 'tier2', 'tier3', 'tier4', 'tier5');
CREATE TYPE stream_type AS ENUM ('rtsp', 'hls', 'youtube', 'dash', 'webrtc', 'mjpeg');
CREATE TYPE stream_protocol AS ENUM ('tcp', 'udp', 'https');
CREATE TYPE video_codec AS ENUM ('h264', 'h265', 'av1', 'vp9');

CREATE TABLE cameras (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name                    TEXT NOT NULL,
    slug                    TEXT UNIQUE NOT NULL,
    camera_type             camera_type NOT NULL DEFAULT 'public',
    external_id             TEXT,

    -- Location: PostGIS geometry replaces lat/lng
    country                 TEXT,
    country_code            CHAR(2),
    region                  TEXT,
    city                    TEXT,
    district                TEXT,
    address                 TEXT,
    custom_label            TEXT,
    location                GEOMETRY(Point, 4326) NOT NULL,
    timezone                TEXT,

    -- Stream (stream_url NEVER exposed in API)
    stream_url              TEXT NOT NULL,
    backup_url              TEXT,
    stream_type             stream_type,
    stream_protocol         stream_protocol,
    stream_quality_default  TEXT,
    available_qualities     JSONB NOT NULL DEFAULT '[]',
    frame_rate              SMALLINT,
    bitrate_kbps            INTEGER,
    codec                   video_codec,
    resolution_w            SMALLINT,
    resolution_h            SMALLINT,
    latency_ms              SMALLINT,

    -- Capabilities
    has_audio               BOOLEAN NOT NULL DEFAULT FALSE,
    has_night_vision        BOOLEAN NOT NULL DEFAULT FALSE,
    is_underwater           BOOLEAN NOT NULL DEFAULT FALSE,

    -- Meta
    category                TEXT NOT NULL DEFAULT 'city',
    subcategory             TEXT,
    tags                    JSONB NOT NULL DEFAULT '[]',
    description_en          TEXT,
    thumbnail_url           TEXT,
    source_url              TEXT,
    attribution             TEXT,

    -- Recording
    recording_enabled       BOOLEAN NOT NULL DEFAULT FALSE,
    retention_tier          retention_tier NOT NULL DEFAULT 'tier5',
    recording_retention_days SMALLINT NOT NULL DEFAULT 0,
    recording_codec         video_codec NOT NULL DEFAULT 'h264',

    -- 3D world fields
    height_above_ground     REAL DEFAULT 5.0,
    camera_azimuth          REAL DEFAULT 0.0,
    camera_elevation        REAL DEFAULT -15.0,
    fov_horizontal          REAL DEFAULT 90.0,
    fov_vertical            REAL DEFAULT 58.0,

    -- Partner / Hardware
    manufacturer            TEXT,
    camera_model            TEXT,
    added_to_placebo_at     TIMESTAMPTZ,
    is_partner_camera       BOOLEAN NOT NULL DEFAULT FALSE,
    owner_name              TEXT,

    -- Timestamps
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ
);

-- Spatial index (CRITICAL for geo queries at scale)
CREATE INDEX idx_cameras_location ON cameras USING GIST (location);

-- Trigram indexes for fuzzy text search
CREATE INDEX idx_cameras_name_trgm ON cameras USING GIN (name gin_trgm_ops);
CREATE INDEX idx_cameras_city_trgm ON cameras USING GIN (city gin_trgm_ops);

-- Standard indexes
CREATE INDEX idx_cameras_slug ON cameras (slug);
CREATE INDEX idx_cameras_city ON cameras (city);
CREATE INDEX idx_cameras_country_code ON cameras (country_code);
CREATE INDEX idx_cameras_category ON cameras (category);
CREATE INDEX idx_cameras_camera_type ON cameras (camera_type);
CREATE INDEX idx_cameras_retention_tier ON cameras (retention_tier);

-- Helper function: extract lat/lng from geometry for API responses
CREATE OR REPLACE FUNCTION camera_lat(g GEOMETRY) RETURNS DOUBLE PRECISION AS $$
    SELECT ST_Y(g);
$$ LANGUAGE SQL IMMUTABLE STRICT;

CREATE OR REPLACE FUNCTION camera_lng(g GEOMETRY) RETURNS DOUBLE PRECISION AS $$
    SELECT ST_X(g);
$$ LANGUAGE SQL IMMUTABLE STRICT;
