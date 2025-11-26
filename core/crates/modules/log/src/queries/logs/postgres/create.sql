INSERT INTO logs (level, module, message, metadata, user_id, session_id, task_id, trace_id, context_id, client_id)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)