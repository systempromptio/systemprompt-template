-- Migration: Add foreign key constraints to user_contexts for data protection
-- This prevents conversations from being orphaned or accidentally deleted
BEGIN;

-- Step 1: Clean up existing orphaned data (set invalid references to NULL)
UPDATE user_contexts
SET session_id = NULL
WHERE session_id IS NOT NULL
  AND NOT EXISTS (SELECT 1 FROM user_sessions WHERE user_sessions.session_id = user_contexts.session_id);

UPDATE user_contexts
SET user_id = NULL
WHERE user_id IS NOT NULL
  AND NOT EXISTS (SELECT 1 FROM users WHERE users.id = user_contexts.user_id);

-- Step 2: Add foreign key for session_id with SET NULL
-- Conversations persist even when sessions expire
ALTER TABLE user_contexts
ADD CONSTRAINT fk_user_contexts_session
    FOREIGN KEY (session_id)
    REFERENCES user_sessions(session_id)
    ON DELETE SET NULL;

-- Step 3: Add foreign key for user_id with SET NULL
-- Conversations persist even when users are deleted (for data retention)
-- Change to RESTRICT if you want to prevent user deletion when conversations exist
ALTER TABLE user_contexts
ADD CONSTRAINT fk_user_contexts_user
    FOREIGN KEY (user_id)
    REFERENCES users(id)
    ON DELETE SET NULL;

COMMIT;
