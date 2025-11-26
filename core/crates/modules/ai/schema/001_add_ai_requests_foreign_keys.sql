-- Migration: Add foreign key constraints to ai_requests for data protection
-- Preserves AI request history even when sessions/contexts expire
BEGIN;

-- Step 1: Clean up existing orphaned data (set invalid references to NULL)
UPDATE ai_requests
SET session_id = NULL
WHERE session_id IS NOT NULL
  AND NOT EXISTS (SELECT 1 FROM user_sessions WHERE user_sessions.session_id = ai_requests.session_id);

UPDATE ai_requests
SET context_id = NULL
WHERE context_id IS NOT NULL
  AND NOT EXISTS (SELECT 1 FROM user_contexts WHERE user_contexts.context_id = ai_requests.context_id);

-- Step 2: Add foreign key for session_id with SET NULL
-- AI requests persist as historical data even when sessions expire
ALTER TABLE ai_requests
ADD CONSTRAINT fk_ai_requests_session
    FOREIGN KEY (session_id)
    REFERENCES user_sessions(session_id)
    ON DELETE SET NULL;

-- Step 3: Add foreign key for context_id with SET NULL
-- AI requests persist even when conversations are deleted
ALTER TABLE ai_requests
ADD CONSTRAINT fk_ai_requests_context
    FOREIGN KEY (context_id)
    REFERENCES user_contexts(context_id)
    ON DELETE SET NULL;

COMMIT;
