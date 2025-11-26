-- Preserve conversations by nullifying session_id instead of deleting
-- This allows conversations to persist even after anonymous sessions expire
UPDATE user_contexts
SET session_id = NULL
WHERE session_id = $1
