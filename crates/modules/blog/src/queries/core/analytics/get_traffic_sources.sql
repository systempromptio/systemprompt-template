SELECT
    COALESCE(
        CASE
            WHEN referrer_source LIKE '%google%' THEN 'Google'
            WHEN referrer_source LIKE '%bing%' THEN 'Bing'
            WHEN referrer_source LIKE '%facebook%' THEN 'Facebook'
            WHEN referrer_source LIKE '%twitter%' OR referrer_source LIKE '%x.com%' THEN 'Twitter/X'
            WHEN referrer_source LIKE '%linkedin%' THEN 'LinkedIn'
            WHEN referrer_source IS NULL THEN 'Direct'
            ELSE referrer_source
        END
    ) as source,
    COUNT(*) as sessions,
    COUNT(DISTINCT user_id) as unique_users,
    ROUND(AVG(duration_seconds), 0) as avg_duration_sec,
    ROUND(AVG(request_count), 1) as avg_pages,
    ROUND(100.0 * SUM(CASE WHEN request_count = 1 THEN 1 ELSE 0 END) / COUNT(*), 1) as bounce_rate_pct
FROM user_sessions
WHERE started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
    AND is_bot = FALSE
    AND is_scanner = FALSE
GROUP BY COALESCE(
        CASE
            WHEN referrer_source LIKE '%google%' THEN 'Google'
            WHEN referrer_source LIKE '%bing%' THEN 'Bing'
            WHEN referrer_source LIKE '%facebook%' THEN 'Facebook'
            WHEN referrer_source LIKE '%twitter%' OR referrer_source LIKE '%x.com%' THEN 'Twitter/X'
            WHEN referrer_source LIKE '%linkedin%' THEN 'LinkedIn'
            WHEN referrer_source IS NULL THEN 'Direct'
            ELSE referrer_source
        END
    )
ORDER BY sessions DESC
LIMIT 20
