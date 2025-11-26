UPDATE agent_tasks
SET status = $1,
    status_timestamp = $2,
    completed_at = CURRENT_TIMESTAMP,
    execution_time_ms = CASE
        WHEN started_at IS NOT NULL THEN
            CAST(EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - started_at)) * 1000 AS INTEGER)
        ELSE
            NULL
    END,
    updated_at = CURRENT_TIMESTAMP
WHERE task_id = $3
