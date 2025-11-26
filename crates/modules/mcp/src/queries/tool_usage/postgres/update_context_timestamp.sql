-- Update context timestamp to trigger SSE change detection
-- Called when tool execution completes with a context_id
--
-- Parameters:
-- $1 = context_id

UPDATE user_contexts
SET updated_at = CURRENT_TIMESTAMP
WHERE context_id = $1
