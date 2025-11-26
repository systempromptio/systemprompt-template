SELECT
    TO_CHAR(DATE_TRUNC('day', started_at), 'YYYY-MM-DD') as date,
    NULL::INTEGER as hour,
    COUNT(CASE WHEN is_bot = false AND is_scanner = false THEN 1 END) as sessions,
    COUNT(DISTINCT CASE WHEN is_bot = false AND is_scanner = false THEN fingerprint_hash END) as unique_visitors,
    SUM(CASE WHEN is_bot = false AND is_scanner = false THEN request_count ELSE 0 END) as pageviews,
    AVG(CASE WHEN is_bot = false AND is_scanner = false THEN duration_seconds END) as avg_session_duration,
    COUNT(CASE WHEN is_bot = true THEN 1 END) as bot_sessions
FROM user_sessions
WHERE started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
GROUP BY date
ORDER BY date DESC
