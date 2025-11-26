UPDATE user_contexts
SET name = $1, updated_at = CURRENT_TIMESTAMP
WHERE context_id = $2 AND user_id = $3
