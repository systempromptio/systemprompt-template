-- The `users` table is owned by core. This extension adds two columns it relies on,
-- plus an index on `department` for the admin filter queries.
ALTER TABLE users ADD COLUMN IF NOT EXISTS department TEXT NOT NULL DEFAULT 'Default';
ALTER TABLE users ADD COLUMN IF NOT EXISTS share_token_version INT NOT NULL DEFAULT 1;
CREATE INDEX IF NOT EXISTS idx_users_department ON users(department);
