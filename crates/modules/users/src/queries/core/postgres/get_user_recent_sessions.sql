SELECT
    s.id as session_id,
    s.user_id,
    u.name as user_name,
    u.email as user_email,
    s.started_at,
    s.last_activity_at,
    s.ended_at,
    s.duration_seconds,
    s.request_count,
    s.ai_request_count,
    s.total_tokens_used,
    s.total_ai_cost_cents,
    s.device_type,
    s.browser,
    s.country
FROM user_sessions s
JOIN users u ON s.user_id = u.id
WHERE s.user_id = $1
ORDER BY s.started_at DESC
LIMIT $2
