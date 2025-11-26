-- Get tasks by user ID (PostgreSQL version)
SELECT * FROM agent_tasks
WHERE user_id = $1
ORDER BY created_at DESC
