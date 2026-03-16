-- Migration 003: Rooms & Room Members

CREATE TABLE rooms (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            TEXT NOT NULL,
    camera_id       UUID REFERENCES cameras(id) ON DELETE SET NULL,
    owner_id        UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    is_private      BOOLEAN NOT NULL DEFAULT FALSE,
    max_members     SMALLINT NOT NULL DEFAULT 4,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_rooms_camera_id ON rooms (camera_id);
CREATE INDEX idx_rooms_owner_id ON rooms (owner_id);

CREATE TABLE room_members (
    room_id         UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    joined_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (room_id, user_id)
);

CREATE INDEX idx_room_members_user_id ON room_members (user_id);
