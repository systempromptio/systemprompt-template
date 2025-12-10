SELECT
    LOWER(REGEXP_REPLACE(REGEXP_REPLACE(COALESCE(referrer_source, 'Direct'), '^https?://(www\.)?', ''), '/$', '')) as referrer_url,
    COUNT(*) as sessions,
    COUNT(DISTINCT user_id) as unique_visitors,
    AVG(request_count) as avg_pages_per_session,
    ROUND(AVG(duration_seconds), 0) as avg_duration_sec
FROM user_sessions
WHERE referrer_source IS NOT NULL
    AND started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
    AND is_bot = FALSE
    AND is_scanner = FALSE
    AND referrer_source NOT IN ('tyingshoelaces.com', 'www.tyingshoelaces.com', 'localhost', 'systemprompt.io', 'www.systemprompt.io')
    AND referrer_source !~ '^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$'
GROUP BY LOWER(REGEXP_REPLACE(REGEXP_REPLACE(COALESCE(referrer_source, 'Direct'), '^https?://(www\.)?', ''), '/$', ''))
HAVING COUNT(*) >= 1
ORDER BY sessions DESC
LIMIT 30
