-- Consolidated schema: Core table amendments
ALTER TABLE users ADD COLUMN IF NOT EXISTS department TEXT NOT NULL DEFAULT 'Default';
CREATE INDEX IF NOT EXISTS idx_users_department ON users(department);
ALTER TABLE users ADD COLUMN IF NOT EXISTS share_token_version INT NOT NULL DEFAULT 1;
