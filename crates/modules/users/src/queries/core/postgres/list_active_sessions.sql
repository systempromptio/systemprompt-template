SELECT
    s.session_id,
    s.user_id,
    u.name as user_name,
    u.email as user_email,
    s.started_at,
    s.last_activity_at,
    s.request_count,
    s.ai_request_count,
    s.total_tokens_used,
    s.device_type,
    s.browser,
    s.country
FROM user_sessions s
LEFT JOIN users u ON s.user_id = u.id
WHERE s.ended_at IS NULL
  AND s.last_activity_at >= CURRENT_TIMESTAMP - INTERVAL '1 hour'
ORDER BY s.last_activity_at DESC
LIMIT 100
