SELECT
    landing_page,
    COUNT(*) as sessions,
    COUNT(DISTINCT fingerprint_hash) as unique_visitors,
    (COUNT(CASE WHEN request_count = 1 THEN 1 END) * 100.0 / NULLIF(COUNT(*), 0)) as bounce_rate,
    AVG(
        CASE
            WHEN ended_at IS NOT NULL THEN duration_seconds
            ELSE EXTRACT(EPOCH FROM (last_activity_at - started_at))::INTEGER
        END
    ) as avg_session_duration
FROM user_sessions
WHERE started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
  AND is_bot = false
  AND is_scanner = false
  AND landing_page IS NOT NULL
  AND request_count > 0
  AND CASE
      WHEN ended_at IS NOT NULL THEN duration_seconds
      ELSE EXTRACT(EPOCH FROM (last_activity_at - started_at))::INTEGER
  END > 0
GROUP BY landing_page
ORDER BY sessions DESC
LIMIT 15
