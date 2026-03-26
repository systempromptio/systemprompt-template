-- Migration: consolidate user_governance into users table
ALTER TABLE users ADD COLUMN IF NOT EXISTS department TEXT NOT NULL DEFAULT '';
CREATE INDEX IF NOT EXISTS idx_users_department ON users(department);
DROP TABLE IF EXISTS user_governance;
