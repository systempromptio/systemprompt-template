SELECT context_id, user_id, name, created_at, updated_at
FROM user_contexts
WHERE context_id = $1 AND user_id = $2
