UPDATE task_messages
SET task_id = $1
WHERE context_id = $2 AND task_id IS NULL
