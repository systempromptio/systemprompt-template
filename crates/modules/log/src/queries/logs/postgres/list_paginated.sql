SELECT id, timestamp, level, module, message, metadata, user_id, session_id, task_id, trace_id, context_id
FROM logs
WHERE
 ($1 IS NULL OR level = $1) AND
 ($2 IS NULL OR module = $2) AND
 ($3 IS NULL OR message LIKE '%' || $3 || '%')
ORDER BY timestamp DESC
LIMIT $4 OFFSET $5