UPDATE user_sessions
SET user_id = $1,
    user_type = 'registered',
    converted_at = CURRENT_TIMESTAMP,
    expires_at = NOW() + INTERVAL '7 days'
WHERE session_id = $2 AND user_type = 'anon'
