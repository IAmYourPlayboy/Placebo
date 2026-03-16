-- Migration 006: Seed test users and sessions (dev only)

INSERT INTO users (id, email, display_name, avatar_url, locale, is_premium, cloud_limit_bytes)
VALUES
    ('a0000000-0000-0000-0000-000000000001', 'alice@placebo.dev', 'Alice', NULL, 'en', TRUE, 53687091200),
    ('a0000000-0000-0000-0000-000000000002', 'bob@placebo.dev', 'Bob', NULL, 'ru', FALSE, 5368709120),
    ('a0000000-0000-0000-0000-000000000003', 'carol@placebo.dev', 'Carol', NULL, 'ja', TRUE, 53687091200)
ON CONFLICT (id) DO NOTHING;

INSERT INTO sessions (id, user_id, token, expires_at)
VALUES
    (gen_random_uuid(), 'a0000000-0000-0000-0000-000000000001', 'dev-token-alice', NOW() + INTERVAL '30 days'),
    (gen_random_uuid(), 'a0000000-0000-0000-0000-000000000002', 'dev-token-bob', NOW() + INTERVAL '30 days'),
    (gen_random_uuid(), 'a0000000-0000-0000-0000-000000000003', 'dev-token-carol', NOW() + INTERVAL '30 days')
ON CONFLICT (token) DO NOTHING;
