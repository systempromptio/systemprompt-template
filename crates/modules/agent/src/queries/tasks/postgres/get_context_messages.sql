-- Get all messages for a context across all tasks
-- Messages are ordered by task start time, then sequence number
SELECT tm.* FROM task_messages tm
INNER JOIN agent_tasks t ON tm.task_id = t.task_id
WHERE t.context_id = $1
ORDER BY t.started_at ASC, tm.sequence_number ASC
