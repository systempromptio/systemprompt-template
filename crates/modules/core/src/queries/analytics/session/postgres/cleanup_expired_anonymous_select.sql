SELECT session_id FROM user_sessions
WHERE user_type = 'anon' AND expires_at < CURRENT_TIMESTAMP
