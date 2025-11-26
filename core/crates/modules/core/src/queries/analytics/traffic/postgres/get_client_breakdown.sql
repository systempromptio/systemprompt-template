SELECT
    COALESCE(client_id, 'unknown') AS client_id,
    COALESCE(client_type, 'unknown') AS client_type,
    COUNT(DISTINCT session_id) AS session_count,
    COUNT(DISTINCT COALESCE(user_id, fingerprint_hash)) AS unique_users
FROM user_sessions
WHERE started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
  AND is_bot = false
  AND is_scanner = false
  AND request_count > 0
GROUP BY client_id, client_type
ORDER BY session_count DESC
