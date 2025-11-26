WITH daily_conversations AS (
    SELECT
        DATE(uc.created_at) AS date,
        COUNT(DISTINCT uc.context_id) AS conversation_count
    FROM user_contexts uc
    WHERE uc.created_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
        AND EXISTS (
            SELECT 1 FROM agent_tasks at
            INNER JOIN task_messages tm ON at.task_id = tm.task_id
            WHERE at.context_id = uc.context_id AND tm.role = 'user'
        )
        AND EXISTS (
            SELECT 1 FROM agent_tasks at2
            INNER JOIN task_messages tm2 ON at2.task_id = tm2.task_id
            WHERE at2.context_id = uc.context_id AND tm2.role = 'agent'
        )
    GROUP BY DATE(uc.created_at)
),
daily_tool_executions AS (
    SELECT
        DATE(started_at) AS date,
        COUNT(*) AS tool_execution_count
    FROM mcp_tool_executions
    WHERE started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
    GROUP BY DATE(started_at)
),
daily_active_users AS (
    SELECT
        DATE(uc.created_at) AS date,
        COUNT(DISTINCT uc.user_id) AS active_user_count
    FROM user_contexts uc
    WHERE uc.created_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
        AND uc.user_id IS NOT NULL
        AND EXISTS (
            SELECT 1 FROM agent_tasks at
            INNER JOIN task_messages tm ON at.task_id = tm.task_id
            WHERE at.context_id = uc.context_id AND tm.role = 'user'
        )
        AND EXISTS (
            SELECT 1 FROM agent_tasks at2
            INNER JOIN task_messages tm2 ON at2.task_id = tm2.task_id
            WHERE at2.context_id = uc.context_id AND tm2.role = 'agent'
        )
    GROUP BY DATE(uc.created_at)
),
date_series AS (
    SELECT generate_series(
        CURRENT_DATE - ($1 || ' days')::INTERVAL,
        CURRENT_DATE,
        '1 day'::INTERVAL
    )::DATE AS date
)
SELECT
    TO_CHAR(ds.date, 'YYYY-MM-DD') AS date,
    COALESCE(dc.conversation_count, 0) AS conversations,
    COALESCE(dte.tool_execution_count, 0) AS tool_executions,
    COALESCE(dau.active_user_count, 0) AS active_users
FROM date_series ds
LEFT JOIN daily_conversations dc ON ds.date = dc.date
LEFT JOIN daily_tool_executions dte ON ds.date = dte.date
LEFT JOIN daily_active_users dau ON ds.date = dau.date
ORDER BY ds.date ASC
