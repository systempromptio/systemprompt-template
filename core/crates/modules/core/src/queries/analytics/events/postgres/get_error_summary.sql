SELECT
    event_type,
    error_code,
    endpoint,
    COUNT(*) as error_count,
    COUNT(DISTINCT session_id) as affected_sessions,
    AVG(response_time_ms) as avg_response_time
FROM analytics_events
WHERE severity IN ('error', 'critical')
AND timestamp >= datetime('now', '-' || $1 || ' hours')
GROUP BY event_type, error_code, endpoint
ORDER BY error_count DESC