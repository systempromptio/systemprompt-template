SELECT
    context_id,
    name,
    updated_at
FROM user_contexts
WHERE user_id = $1 AND context_id = $2 AND updated_at > $3
