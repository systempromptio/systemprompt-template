-- Get task status changes since last seen timestamp
-- Used by SSE stream to send delta updates for TaskStatusChanged events
--
-- Parameters:
-- $1 = user_id (from context lookup)
-- $2 = context_id
-- $3 = last_seen_timestamp

SELECT
    at.task_id,
    at.context_id,
    at.status as status,
    at.updated_at as timestamp
FROM agent_tasks at
WHERE at.context_id = $2
  AND at.updated_at > $3
ORDER BY at.updated_at ASC;
