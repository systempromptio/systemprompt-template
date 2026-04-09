-- Consolidated schema: Core table amendments
ALTER TABLE users ADD COLUMN IF NOT EXISTS department TEXT NOT NULL DEFAULT '';
CREATE INDEX IF NOT EXISTS idx_users_department ON users(department);
