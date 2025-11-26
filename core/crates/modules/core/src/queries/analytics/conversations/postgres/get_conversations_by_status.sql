WITH latest_tasks AS (
    SELECT DISTINCT ON (at.context_id)
        at.context_id,
        at.status
    FROM agent_tasks at
    WHERE at.created_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
        AND EXISTS (
            SELECT 1 FROM agent_tasks at2
            INNER JOIN task_messages tm ON at2.task_id = tm.task_id
            WHERE at2.context_id = at.context_id AND tm.role = 'user'
        )
        AND EXISTS (
            SELECT 1 FROM agent_tasks at3
            INNER JOIN task_messages tm2 ON at3.task_id = tm2.task_id
            WHERE at3.context_id = at.context_id AND tm2.role = 'agent'
        )
    ORDER BY at.context_id, at.created_at DESC
)
SELECT
    status,
    COUNT(*) AS count
FROM latest_tasks
GROUP BY status
ORDER BY count DESC
