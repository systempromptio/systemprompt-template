-- Bot Traffic Timeline
-- Returns daily bot/scanner activity trends
SELECT
    DATE(started_at) as date,
    SUM(CASE WHEN is_bot = TRUE THEN 1 ELSE 0 END) as bot_sessions,
    SUM(CASE WHEN is_scanner = TRUE THEN 1 ELSE 0 END) as scanner_sessions,
    COUNT(DISTINCT ip_address) as unique_ips,
    SUM(request_count) as total_requests,
    ROUND(AVG(request_count), 2) as avg_requests_per_session,
    ROUND(AVG(
        CASE
            WHEN ended_at IS NOT NULL THEN
                EXTRACT(EPOCH FROM (ended_at - started_at))::INTEGER
            ELSE
                EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - started_at))::INTEGER
        END
    ), 2) as avg_duration_seconds
FROM user_sessions
WHERE started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
  AND (is_bot = TRUE OR is_scanner = TRUE)
GROUP BY DATE(started_at)
ORDER BY date DESC;
