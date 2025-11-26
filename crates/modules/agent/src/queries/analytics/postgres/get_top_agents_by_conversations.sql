SELECT
    agent_name,
    COUNT(*) AS conversation_count,
    ROW_NUMBER() OVER (ORDER BY COUNT(*) DESC) AS rank
FROM agent_tasks
WHERE created_at > CURRENT_TIMESTAMP - INTERVAL '7 days'
GROUP BY agent_name
ORDER BY conversation_count DESC
LIMIT 5
