-- List all tasks for a specific context (PostgreSQL version)
SELECT * FROM agent_tasks
WHERE context_id = $1
ORDER BY created_at ASC
