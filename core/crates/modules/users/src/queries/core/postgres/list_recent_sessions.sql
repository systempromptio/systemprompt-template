SELECT
    s.session_id,
    s.user_id,
    u.name AS user_name,
    u.email AS user_email,
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
LEFT JOIN users u ON s.user_id = u.id
ORDER BY s.started_at DESC
LIMIT 100
