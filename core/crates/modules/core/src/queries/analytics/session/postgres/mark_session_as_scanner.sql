UPDATE user_sessions
SET is_scanner = TRUE
WHERE session_id = $1
