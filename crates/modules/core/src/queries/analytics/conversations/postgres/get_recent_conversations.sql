WITH task_messages_count AS (
    SELECT task_id, COUNT(*) AS message_count
    FROM task_messages
    GROUP BY task_id
    HAVING COUNT(*) >= 2
),
recent_valid_tasks AS (
    SELECT DISTINCT ON (at.context_id)
        at.context_id,
        at.task_id,
        at.agent_name,
        at.status,
        at.created_at AS task_started_at,
        at.updated_at AS task_completed_at,
        tmc.message_count
    FROM agent_tasks at
    INNER JOIN task_messages_count tmc ON at.task_id = tmc.task_id
    WHERE at.agent_name IS NOT NULL
        AND at.created_at IS NOT NULL
        AND at.updated_at IS NOT NULL
    ORDER BY at.context_id, at.created_at DESC
)
SELECT
    uc.context_id,
    rt.agent_name,
    uc.created_at AS context_started_at,
    rt.task_started_at,
    rt.task_completed_at,
    rt.status,
    rt.message_count
FROM user_contexts uc
INNER JOIN recent_valid_tasks rt ON uc.context_id = rt.context_id
WHERE uc.created_at >= CURRENT_TIMESTAMP - INTERVAL '7 days'
ORDER BY uc.created_at DESC
LIMIT $1
