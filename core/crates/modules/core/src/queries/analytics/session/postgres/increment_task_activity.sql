UPDATE user_sessions
SET
    task_count = task_count + $1,
    message_count = message_count + $2
WHERE session_id = $3