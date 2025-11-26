SELECT
    COUNT(DISTINCT CASE WHEN uc.created_at >= CURRENT_TIMESTAMP - INTERVAL '24 hours' THEN uc.context_id END) AS conversations_24h,
    COUNT(DISTINCT CASE WHEN uc.created_at >= CURRENT_TIMESTAMP - INTERVAL '7 days' THEN uc.context_id END) AS conversations_7d,
    COUNT(DISTINCT CASE WHEN uc.created_at >= CURRENT_TIMESTAMP - INTERVAL '30 days' THEN uc.context_id END) AS conversations_30d,
    COUNT(DISTINCT CASE WHEN uc.created_at >= CURRENT_TIMESTAMP - INTERVAL '48 hours' AND uc.created_at < CURRENT_TIMESTAMP - INTERVAL '24 hours' THEN uc.context_id END) AS conversations_prev_24h,
    COUNT(DISTINCT CASE WHEN uc.created_at >= CURRENT_TIMESTAMP - INTERVAL '14 days' AND uc.created_at < CURRENT_TIMESTAMP - INTERVAL '7 days' THEN uc.context_id END) AS conversations_prev_7d,
    COUNT(DISTINCT CASE WHEN uc.created_at >= CURRENT_TIMESTAMP - INTERVAL '60 days' AND uc.created_at < CURRENT_TIMESTAMP - INTERVAL '30 days' THEN uc.context_id END) AS conversations_prev_30d
FROM user_contexts uc
WHERE EXISTS (
    SELECT 1 FROM agent_tasks at
    INNER JOIN task_messages tm ON at.task_id = tm.task_id
    WHERE at.context_id = uc.context_id AND tm.role = 'user'
)
AND EXISTS (
    SELECT 1 FROM agent_tasks at2
    INNER JOIN task_messages tm2 ON at2.task_id = tm2.task_id
    WHERE at2.context_id = uc.context_id AND tm2.role = 'agent'
)
