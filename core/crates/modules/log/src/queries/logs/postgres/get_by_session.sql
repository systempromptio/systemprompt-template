SELECT id, timestamp, level, module, message, metadata, user_id, session_id, task_id, trace_id, context_id
FROM logs
WHERE session_id = $1
ORDER BY timestamp DESC
LIMIT $2 OFFSET $3
