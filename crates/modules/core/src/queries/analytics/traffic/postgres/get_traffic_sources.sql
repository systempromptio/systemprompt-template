SELECT
    COALESCE(
        CASE
            WHEN referrer_source IS NULL THEN 'Direct'
            WHEN referrer_source ILIKE '%google%' THEN 'Google'
            WHEN referrer_source ILIKE '%bing%' THEN 'Bing'
            WHEN referrer_source ILIKE '%facebook%' THEN 'Facebook'
            WHEN referrer_source ILIKE '%twitter%' OR referrer_source ILIKE '%t.co%' THEN 'Twitter'
            WHEN referrer_source ILIKE '%linkedin%' THEN 'LinkedIn'
            ELSE referrer_source
        END,
        'Direct'
    ) as source_name,
    COUNT(*) as session_count,
    COUNT(DISTINCT fingerprint_hash) as unique_visitors,
    AVG(
        CASE
            WHEN ended_at IS NOT NULL THEN duration_seconds
            ELSE EXTRACT(EPOCH FROM (last_activity_at - started_at))::INTEGER
        END
    ) as avg_engagement_seconds,
    (COUNT(CASE WHEN request_count = 1 THEN 1 END) * 100.0 / NULLIF(COUNT(*), 0)) as bounce_rate
FROM user_sessions
WHERE started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
  AND is_bot = false
  AND is_scanner = false
  AND request_count > 0
  AND CASE
      WHEN ended_at IS NOT NULL THEN duration_seconds
      ELSE EXTRACT(EPOCH FROM (last_activity_at - started_at))::INTEGER
  END > 0
GROUP BY referrer_source
ORDER BY session_count DESC
LIMIT 10
