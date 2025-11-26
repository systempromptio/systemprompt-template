UPDATE agent_tasks
SET status = $1,
    status_timestamp = $2,
    started_at = COALESCE(started_at, CURRENT_TIMESTAMP),
    updated_at = CURRENT_TIMESTAMP
WHERE task_id = $3
