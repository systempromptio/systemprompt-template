-- Migration: Add bot tracking to users table
-- This allows analytics to work even after sessions expire
-- Users persist, sessions don't - bot status should be on users
BEGIN;

-- Step 1: Add bot tracking columns to users
ALTER TABLE users
ADD COLUMN is_bot BOOLEAN DEFAULT FALSE NOT NULL,
ADD COLUMN is_scanner BOOLEAN DEFAULT FALSE NOT NULL;

-- Step 2: Backfill bot status from existing session data
-- Mark user as bot if ANY of their sessions were bots
UPDATE users u
SET is_bot = TRUE
WHERE EXISTS (
    SELECT 1 FROM user_sessions s
    WHERE s.user_id = u.id
      AND s.is_bot = TRUE
);

-- Mark user as scanner if ANY of their sessions were scanners
UPDATE users u
SET is_scanner = TRUE
WHERE EXISTS (
    SELECT 1 FROM user_sessions s
    WHERE s.user_id = u.id
      AND s.is_scanner = TRUE
);

-- Step 3: Create indexes for analytics queries
CREATE INDEX idx_users_bot_status ON users(is_bot, is_scanner);

COMMIT;
