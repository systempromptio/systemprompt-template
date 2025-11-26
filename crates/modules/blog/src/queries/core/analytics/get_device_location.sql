SELECT
    COALESCE(device_type, 'Unknown') as device,
    COALESCE(country, 'Unknown') as country,
    COUNT(*) as sessions,
    ROUND(100.0 * COUNT(*) / SUM(COUNT(*)) OVER (), 1) as pct_of_total,
    COUNT(DISTINCT user_id) as unique_users,
    ROUND(AVG(request_count), 1) as avg_pages
FROM user_sessions
WHERE started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
    AND is_bot = FALSE
    AND is_scanner = FALSE
GROUP BY device_type, country
ORDER BY sessions DESC
LIMIT 50
