INSERT INTO context_agents (context_id, agent_name, last_active_at)
VALUES ($1, $2, CURRENT_TIMESTAMP)
ON CONFLICT(context_id, agent_name)
DO UPDATE SET last_active_at = CURRENT_TIMESTAMP
