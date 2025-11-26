SELECT
    COALESCE(device_type, 'unknown') AS device_type,
    COUNT(DISTINCT session_id) AS session_count,
    SUM(COALESCE(request_count, 0)) AS request_count
FROM user_sessions
WHERE started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
  AND is_bot = false
  AND is_scanner = false
  AND request_count > 0
GROUP BY device_type
ORDER BY session_count DESC
