-- Get recent conversations with pagination and evaluation data
-- For dashboard display with summary cards
-- Only includes conversations with at least one user message AND one agent message
-- Uses LATERAL subquery to select the most recent task per context (deterministic behavior)
SELECT
    uc.context_id,
    uc.name AS conversation_name,
    uc.user_id,
    COALESCE(u.name, 'anonymous') AS user_name,
    COALESCE(at.agent_name, 'Unknown') AS agent_name,
    at.started_at,
    to_char(at.started_at, 'Mon DD, HH12:MI AM') AS started_at_formatted,
    CASE
        WHEN at.completed_at IS NOT NULL THEN
            EXTRACT(EPOCH FROM (at.completed_at - at.started_at))::FLOAT
        ELSE
            EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - at.started_at))::FLOAT
    END AS duration_seconds,
    CASE
        WHEN at.completed_at IS NOT NULL THEN 'completed'::text
        ELSE 'active'::text
    END AS duration_status,
    COALESCE(at.status, 'unknown') AS status,
    COALESCE(context_msg_count.message_count, 0) AS message_count,
    COALESCE(ce.conversation_quality, NULL) AS quality_score,
    COALESCE(ce.goal_achieved, NULL) AS goal_achieved,
    COALESCE(ce.user_satisfied, NULL) AS user_satisfaction,
    COALESCE(ce.primary_category, NULL) AS primary_category,
    COALESCE(ce.topics_discussed, NULL) AS topics,
    COALESCE(ce.evaluation_summary, NULL) AS evaluation_summary
FROM user_contexts uc
LEFT JOIN users u ON uc.user_id = u.id
LEFT JOIN LATERAL (
    SELECT task_id, agent_name, status, started_at, completed_at
    FROM agent_tasks
    WHERE context_id = uc.context_id
    ORDER BY created_at DESC
    LIMIT 1
) AS at ON true
LEFT JOIN (
    SELECT at2.context_id, COUNT(tm.id) AS message_count
    FROM agent_tasks at2
    INNER JOIN task_messages tm ON at2.task_id = tm.task_id
    GROUP BY at2.context_id
) AS context_msg_count ON uc.context_id = context_msg_count.context_id
LEFT JOIN conversation_evaluations ce ON uc.context_id = ce.context_id
WHERE uc.created_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
    AND EXISTS (
        SELECT 1 FROM agent_tasks at2
        INNER JOIN task_messages tm ON at2.task_id = tm.task_id
        WHERE at2.context_id = uc.context_id
            AND tm.role = 'user'
    )
    AND EXISTS (
        SELECT 1 FROM agent_tasks at3
        INNER JOIN task_messages tm2 ON at3.task_id = tm2.task_id
        WHERE at3.context_id = uc.context_id
            AND tm2.role = 'agent'
    )
ORDER BY uc.created_at DESC
LIMIT $2 OFFSET $3;
