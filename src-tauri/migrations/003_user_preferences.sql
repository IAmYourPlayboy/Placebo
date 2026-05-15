CREATE TABLE IF NOT EXISTS user_preferences (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Seed defaults; INSERT OR IGNORE to avoid overwriting user choices.
INSERT OR IGNORE INTO user_preferences (key, value) VALUES
    ('theme', 'auto'),
    ('lang',  'ru');
