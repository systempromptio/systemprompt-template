WITH task_stats AS (
    SELECT
        at.context_id,
        at.task_id,
        at.status,
        at.execution_time_ms
    FROM agent_tasks at
    WHERE at.created_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
),
context_messages AS (
    SELECT
        ts.context_id,
        COUNT(tm.id)::INTEGER AS message_count,
        AVG(ts.execution_time_ms) FILTER (WHERE ts.execution_time_ms IS NOT NULL) AS avg_execution_time_ms,
        COUNT(DISTINCT CASE WHEN ts.status = 'failed' THEN ts.task_id END) > 0 AS has_failed_tasks
    FROM task_stats ts
    LEFT JOIN task_messages tm ON ts.task_id = tm.task_id
    GROUP BY ts.context_id
    HAVING COUNT(tm.id) > 0
        AND EXISTS (
            SELECT 1 FROM agent_tasks at2
            INNER JOIN task_messages tm2 ON at2.task_id = tm2.task_id
            WHERE at2.context_id = ts.context_id AND tm2.role = 'user'
        )
        AND EXISTS (
            SELECT 1 FROM agent_tasks at3
            INNER JOIN task_messages tm3 ON at3.task_id = tm3.task_id
            WHERE at3.context_id = ts.context_id AND tm3.role = 'agent'
        )
)
SELECT
    COUNT(*) AS total_conversations,
    COALESCE(SUM(message_count), 0)::BIGINT AS total_messages,
    CASE
        WHEN COUNT(*) > 0
        THEN SUM(message_count)::DOUBLE PRECISION / COUNT(*)
        ELSE 0
    END AS avg_messages_per_conversation,
    COALESCE(AVG(COALESCE(avg_execution_time_ms, 0)::DOUBLE PRECISION), 0) AS avg_execution_time_ms,
    COUNT(*) FILTER (WHERE has_failed_tasks) AS failed_conversations
FROM context_messages
