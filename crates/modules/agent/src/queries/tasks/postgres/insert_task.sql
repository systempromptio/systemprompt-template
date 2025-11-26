INSERT INTO agent_tasks (
    task_id, context_id, status, status_timestamp,
    user_id, session_id, trace_id, metadata, agent_name, started_at
) VALUES ($1, $2, $3, $4, $5, $6, $7, $8::jsonb, $9, CURRENT_TIMESTAMP)
ON CONFLICT(task_id) DO UPDATE SET
    status = excluded.status,
    status_timestamp = excluded.status_timestamp,
    agent_name = excluded.agent_name,
    started_at = COALESCE(agent_tasks.started_at, excluded.started_at),
    updated_at = CURRENT_TIMESTAMP
