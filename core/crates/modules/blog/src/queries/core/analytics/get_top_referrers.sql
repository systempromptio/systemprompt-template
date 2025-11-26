SELECT
    referrer_source,
    referrer_url,
    COUNT(*) as sessions,
    COUNT(DISTINCT user_id) as unique_visitors,
    AVG(request_count) as avg_pages_per_session,
    ROUND(AVG(duration_seconds), 0) as avg_duration_sec
FROM user_sessions
WHERE referrer_url IS NOT NULL
    AND started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
    AND is_bot = FALSE
    AND is_scanner = FALSE
GROUP BY referrer_source, referrer_url
HAVING COUNT(*) >= 2
ORDER BY sessions DESC
LIMIT 30
