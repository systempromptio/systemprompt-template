SELECT
    COALESCE(country, 'unknown') AS country,
    COUNT(DISTINCT session_id) AS session_count,
    SUM(COALESCE(request_count, 0)) AS request_count
FROM user_sessions
WHERE started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
    AND country IS NOT NULL
    AND is_bot = false
    AND is_scanner = false
    AND request_count > 0
GROUP BY country
ORDER BY session_count DESC
LIMIT 20
