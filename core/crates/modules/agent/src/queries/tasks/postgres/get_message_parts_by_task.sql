SELECT * FROM message_parts
WHERE message_id = $1 AND task_id = $2
ORDER BY sequence_number ASC
