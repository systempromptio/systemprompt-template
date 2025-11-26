-- Suspicious IPs
-- Returns IPs making suspicious requests with threat classification
SELECT
    ip_address,
    COUNT(DISTINCT session_id) as session_count,
    COUNT(DISTINCT user_agent) as user_agent_variants,
    SUM(request_count) as total_requests,
    SUM(CASE WHEN is_bot = TRUE THEN 1 ELSE 0 END) as bot_sessions,
    SUM(CASE WHEN is_scanner = TRUE THEN 1 ELSE 0 END) as scanner_sessions,
    country,
    region,
    city,
    MIN(started_at) as first_seen,
    MAX(last_activity_at) as last_seen,
    CASE
        WHEN SUM(request_count) > 1000 THEN 'high_volume_attack'
        WHEN COUNT(DISTINCT user_agent) > 5 THEN 'distributed_attack'
        WHEN SUM(CASE WHEN is_scanner = TRUE THEN 1 ELSE 0 END) > 3 THEN 'scanning_campaign'
        ELSE 'suspicious_activity'
    END as threat_level
FROM user_sessions
WHERE started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
  AND (is_bot = TRUE OR is_scanner = TRUE)
GROUP BY ip_address, country, region, city
ORDER BY total_requests DESC
LIMIT 100;
