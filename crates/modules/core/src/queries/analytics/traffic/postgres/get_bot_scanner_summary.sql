SELECT
    COUNT(DISTINCT CASE WHEN is_bot = TRUE THEN session_id END) AS bot_sessions,
    COUNT(DISTINCT CASE WHEN is_scanner = TRUE THEN session_id END) AS scanner_sessions,
    COUNT(DISTINCT CASE WHEN is_bot = TRUE OR is_scanner = TRUE THEN session_id END) AS total_automated_sessions,
    SUM(CASE WHEN is_bot = TRUE THEN request_count ELSE 0 END) AS bot_requests,
    SUM(CASE WHEN is_scanner = TRUE THEN request_count ELSE 0 END) AS scanner_requests,
    COUNT(DISTINCT session_id) AS total_all_sessions,
    (COUNT(DISTINCT CASE WHEN is_bot = TRUE OR is_scanner = TRUE THEN session_id END) * 100.0 /
     NULLIF(COUNT(DISTINCT session_id), 0)) AS automated_percentage
FROM user_sessions
WHERE started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
  AND request_count > 0
