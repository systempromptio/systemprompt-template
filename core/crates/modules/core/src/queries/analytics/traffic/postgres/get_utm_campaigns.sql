SELECT
    COALESCE(utm_campaign, '(not set)') as campaign_name,
    COALESCE(utm_source, '(not set)') as source,
    COALESCE(utm_medium, '(not set)') as medium,
    COUNT(*) as sessions,
    COUNT(DISTINCT fingerprint_hash) as unique_visitors,
    AVG(
        CASE
            WHEN ended_at IS NOT NULL THEN duration_seconds
            ELSE EXTRACT(EPOCH FROM (last_activity_at - started_at))::INTEGER
        END
    ) as avg_engagement,
    (COUNT(CASE WHEN task_count > 0 THEN 1 END) * 100.0 / NULLIF(COUNT(*), 0)) as conversion_rate
FROM user_sessions
WHERE started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
  AND is_bot = false
  AND is_scanner = false
  AND (utm_campaign IS NOT NULL OR utm_source IS NOT NULL OR utm_medium IS NOT NULL)
  AND request_count > 0
  AND CASE
      WHEN ended_at IS NOT NULL THEN duration_seconds
      ELSE EXTRACT(EPOCH FROM (last_activity_at - started_at))::INTEGER
  END > 0
GROUP BY utm_campaign, utm_source, utm_medium
ORDER BY sessions DESC
LIMIT 20
