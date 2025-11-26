SELECT id, timestamp, level, module, message, metadata, user_id, session_id, task_id, trace_id, context_id
FROM (
    SELECT id, timestamp, level, module, message, metadata, user_id, session_id, task_id, trace_id, context_id
    FROM logs
    WHERE module = $1
    ORDER BY timestamp DESC
    LIMIT $2
)
ORDER BY timestamp ASC