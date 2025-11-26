-- Get tasks by user ID with pagination (PostgreSQL version)
SELECT * FROM agent_tasks
WHERE user_id = $1
ORDER BY created_at DESC
LIMIT $2 OFFSET $3
