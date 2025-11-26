SELECT
    u.id AS user_id,
    u.name,
    u.email,
    COUNT(DISTINCT s.session_id) AS total_sessions,
    SUM(s.request_count) AS total_requests,
    SUM(s.ai_request_count) AS total_ai_requests,
    SUM(s.total_tokens_used) AS total_tokens,
    SUM(s.total_ai_cost_cents) AS total_cost_cents,
    COUNT(DISTINCT t.uuid) AS total_tasks,
    COUNT(DISTINCT t.context_id) AS total_contexts,
    MAX(s.last_activity_at) AS last_active
FROM users u
LEFT JOIN user_sessions s ON u.id = s.user_id
    AND s.started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
LEFT JOIN agent_tasks t ON u.id = t.user_id
    AND t.created_at >= CURRENT_TIMESTAMP - ($2 || ' days')::INTERVAL
WHERE u.id = $3
GROUP BY u.id
