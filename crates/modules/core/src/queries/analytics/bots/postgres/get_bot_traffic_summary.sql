-- Bot Traffic Summary
-- Returns aggregated metrics for bot and scanner activity
SELECT
    COALESCE(user_agent, 'unknown') as bot_name,
    SUM(CASE WHEN is_bot = TRUE THEN 1 ELSE 0 END) as bot_sessions,
    SUM(CASE WHEN is_scanner = TRUE THEN 1 ELSE 0 END) as scanner_sessions,
    COUNT(DISTINCT ip_address) as unique_ips,
    SUM(request_count) as total_requests,
    ROUND(AVG(request_count), 2) as avg_requests_per_session,
    MIN(started_at) as first_seen,
    MAX(last_activity_at) as last_seen,
    SUM(CASE WHEN is_bot = TRUE THEN 1 ELSE 0 END) + SUM(CASE WHEN is_scanner = TRUE THEN 1 ELSE 0 END) as total_suspicious_sessions
FROM user_sessions
WHERE started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
  AND (is_bot = TRUE OR is_scanner = TRUE)
GROUP BY user_agent
ORDER BY total_suspicious_sessions DESC
LIMIT 50;
