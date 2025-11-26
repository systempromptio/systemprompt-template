SELECT * FROM task_messages
WHERE task_id = $1
ORDER BY sequence_number ASC
