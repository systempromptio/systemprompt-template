INSERT INTO agent_tasks (task_id, context_id, status, status_timestamp, user_id, session_id, trace_id, metadata, agent_name, started_at)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, CURRENT_TIMESTAMP)
