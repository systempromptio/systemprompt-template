SELECT
    DATE(s.started_at) as date,
    COUNT(DISTINCT s.user_id) as daily_active_users,
    COUNT(s.session_id) as new_sessions,
    SUM(s.request_count) as total_requests
FROM user_sessions s
WHERE s.started_at >= NOW() - ($1::INTEGER || ' days')::INTERVAL
GROUP BY DATE(s.started_at)
ORDER BY date ASC;
