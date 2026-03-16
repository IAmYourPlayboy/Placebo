-- Placebo: Camera table
-- All fields from PLACEBO_CLAUDE_CODE_CONTEXT.md Section 9

CREATE TABLE IF NOT EXISTS cameras (
    -- ИДЕНТИФИКАЦИЯ
    id                      TEXT PRIMARY KEY,
    name                    TEXT NOT NULL,
    slug                    TEXT UNIQUE NOT NULL,
    camera_type             TEXT NOT NULL DEFAULT 'public' CHECK (camera_type IN ('public', 'enterprise', 'yourself')),
    external_id             TEXT,

    -- ЛОКАЦИЯ
    country                 TEXT,
    country_code            TEXT,
    region                  TEXT,
    city                    TEXT,
    district                TEXT,
    address                 TEXT,
    custom_label            TEXT,
    lat                     REAL NOT NULL,
    lng                     REAL NOT NULL,
    timezone                TEXT,

    -- СТРИМ
    stream_url              TEXT NOT NULL,
    backup_url              TEXT,
    stream_type             TEXT CHECK (stream_type IN ('rtsp', 'hls', 'youtube', 'dash', 'webrtc', 'mjpeg')),
    stream_protocol         TEXT CHECK (stream_protocol IN ('tcp', 'udp', 'https')),
    stream_quality_default  TEXT,
    available_qualities     TEXT DEFAULT '[]',
    frame_rate              INTEGER,
    bitrate_kbps            INTEGER,
    codec                   TEXT CHECK (codec IN ('h264', 'h265', 'av1', 'vp9')),
    resolution_w            INTEGER,
    resolution_h            INTEGER,
    latency_ms              INTEGER,

    -- ВОЗМОЖНОСТИ
    has_audio               INTEGER NOT NULL DEFAULT 0,
    has_night_vision        INTEGER NOT NULL DEFAULT 0,
    is_underwater           INTEGER NOT NULL DEFAULT 0,

    -- МЕТА
    category                TEXT NOT NULL DEFAULT 'city',
    subcategory             TEXT,
    tags                    TEXT NOT NULL DEFAULT '[]',
    description_en          TEXT,
    thumbnail_url           TEXT,
    source_url              TEXT,
    attribution             TEXT,

    -- ЗАПИСЬ
    recording_enabled       INTEGER NOT NULL DEFAULT 0,
    retention_tier          TEXT NOT NULL DEFAULT 'tier5' CHECK (retention_tier IN ('tier1', 'tier2', 'tier3', 'tier4', 'tier5')),
    recording_retention_days INTEGER NOT NULL DEFAULT 0,
    recording_codec         TEXT NOT NULL DEFAULT 'h264',

    -- ПАРТНЁР / ЖЕЛЕЗО
    manufacturer            TEXT,
    camera_model            TEXT,
    added_to_placebo_at     TEXT,
    is_partner_camera       INTEGER NOT NULL DEFAULT 0,
    owner_name              TEXT,

    -- TIMESTAMPS
    created_at              TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at              TEXT
);

-- Индексы
CREATE UNIQUE INDEX IF NOT EXISTS idx_cameras_slug ON cameras(slug);
CREATE INDEX IF NOT EXISTS idx_cameras_city ON cameras(city);
CREATE INDEX IF NOT EXISTS idx_cameras_country_code ON cameras(country_code);
CREATE INDEX IF NOT EXISTS idx_cameras_category ON cameras(category);
CREATE INDEX IF NOT EXISTS idx_cameras_camera_type ON cameras(camera_type);
CREATE INDEX IF NOT EXISTS idx_cameras_location ON cameras(lat, lng);
