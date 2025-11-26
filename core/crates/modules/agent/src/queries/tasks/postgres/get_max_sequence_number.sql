SELECT MAX(sequence_number) as max_seq
FROM task_messages
WHERE task_id = $1
