SELECT agent_name
FROM agent_tasks
WHERE context_id = $1
ORDER BY created_at DESC
LIMIT 1
