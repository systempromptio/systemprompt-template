UPDATE agent_tasks
SET status = $1,
    status_timestamp = $2,
    metadata = $3::jsonb,
    started_at = COALESCE(started_at, CURRENT_TIMESTAMP),
    completed_at = CURRENT_TIMESTAMP,
    execution_time_ms = CAST(EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - COALESCE(started_at, CURRENT_TIMESTAMP))) * 1000 AS INTEGER),
    updated_at = CURRENT_TIMESTAMP
WHERE task_id = $4
