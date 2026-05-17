-- Migration 008: Username + Date of Birth on users
--
-- Adds:
--   username                – chosen handle (Latin alnum + underscore, 3-24 chars, app-validated)
--   username_normalized     – generated column (lower(username)) used for case-insensitive uniqueness
--   date_of_birth           – optional NaiveDate
--   date_of_birth_hidden    – default TRUE; users opt in to showing DOB on public profile
--
-- Backfill: existing rows get a synthetic username derived from the email prefix plus a 6-char
-- slice of the user's UUID, so the unique index never collides on legacy data.

ALTER TABLE users
    ADD COLUMN username              TEXT,
    ADD COLUMN username_normalized   TEXT GENERATED ALWAYS AS (lower(username)) STORED,
    ADD COLUMN date_of_birth         DATE,
    ADD COLUMN date_of_birth_hidden  BOOLEAN NOT NULL DEFAULT TRUE;

-- Case-insensitive uniqueness on the normalised column. NULLs are allowed so legacy/migrated
-- rows that somehow lack a username don't break the constraint; new registrations are required
-- to provide one at the application layer.
CREATE UNIQUE INDEX idx_users_username_normalized
    ON users (username_normalized)
    WHERE username_normalized IS NOT NULL;

-- Backfill existing dev/seed users with `<emailprefix>_<uuid6>` so each row has a username.
UPDATE users
SET username = split_part(email, '@', 1) || '_' || substring(id::text, 1, 6)
WHERE username IS NULL;
