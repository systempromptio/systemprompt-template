-- Fast query for context initialization
-- Returns only basic context data without expensive aggregations
-- Used for immediate SSE snapshot to unblock frontend initialization

SELECT
    context_id,
    user_id,
    name,
    created_at,
    updated_at
FROM user_contexts
WHERE user_id = $1
ORDER BY updated_at DESC
