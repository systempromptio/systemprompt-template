SELECT id, timestamp, level, module, message, metadata, user_id, session_id, task_id, trace_id, context_id
FROM logs
WHERE id = $1