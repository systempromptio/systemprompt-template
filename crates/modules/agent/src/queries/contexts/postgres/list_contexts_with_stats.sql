SELECT
    uc.context_id,
    uc.user_id,
    uc.name,
    uc.created_at,
    uc.updated_at,
    COUNT(DISTINCT at.task_id) as task_count,
    COUNT(DISTINCT tm.id) as message_count,
    MAX(tm.created_at) as last_message_at
FROM user_contexts uc
LEFT JOIN agent_tasks at ON uc.context_id = at.context_id
LEFT JOIN task_messages tm ON at.task_id = tm.task_id AND (tm.user_id = uc.user_id OR tm.user_id IS NULL)
WHERE uc.user_id = $1
GROUP BY uc.context_id, uc.user_id, uc.name, uc.created_at, uc.updated_at
ORDER BY uc.updated_at DESC
