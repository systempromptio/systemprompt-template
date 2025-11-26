SELECT COUNT(*) as session_count
FROM user_sessions
WHERE client_id = $1
  AND started_at >= CURRENT_TIMESTAMP - INTERVAL '1 hour'
