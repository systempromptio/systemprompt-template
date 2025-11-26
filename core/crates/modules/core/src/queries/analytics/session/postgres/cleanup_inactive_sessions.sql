UPDATE user_sessions
SET
    ended_at = last_activity_at,
    duration_seconds = EXTRACT(EPOCH FROM (last_activity_at - started_at))::INTEGER
WHERE ended_at IS NULL
  AND last_activity_at < CURRENT_TIMESTAMP - ($1 || ' hours')::INTERVAL