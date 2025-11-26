UPDATE user_sessions
SET
    ended_at = CURRENT_TIMESTAMP,
    duration_seconds = EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - started_at))::INTEGER
WHERE session_id = $1 AND ended_at IS NULL