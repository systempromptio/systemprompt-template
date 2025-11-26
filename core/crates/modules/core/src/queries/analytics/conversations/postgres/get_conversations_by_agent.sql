-- Count conversations (contexts) by agent
-- Each context can have multiple agents
-- We count distinct contexts per agent (only those with user AND agent messages)
WITH agent_contexts AS (
    SELECT
        DISTINCT at.agent_name,
        at.context_id
    FROM agent_tasks at
    INNER JOIN task_messages tm ON at.task_id = tm.task_id
    WHERE at.created_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
        AND at.agent_name IS NOT NULL
        AND at.agent_name != ''
        AND EXISTS (
            SELECT 1 FROM agent_tasks at2
            INNER JOIN task_messages tm2 ON at2.task_id = tm2.task_id
            WHERE at2.context_id = at.context_id AND tm2.role = 'user'
        )
        AND EXISTS (
            SELECT 1 FROM agent_tasks at3
            INNER JOIN task_messages tm3 ON at3.task_id = tm3.task_id
            WHERE at3.context_id = at.context_id AND tm3.role = 'agent'
        )
),
agent_conversation_counts AS (
    SELECT
        agent_name,
        COUNT(DISTINCT context_id) AS conversation_count
    FROM agent_contexts
    GROUP BY agent_name
),
total_conversations AS (
    SELECT COUNT(DISTINCT context_id) AS total
    FROM agent_contexts
)
SELECT
    ac.agent_name,
    ac.conversation_count,
    CASE
        WHEN tc.total > 0
        THEN (ac.conversation_count * 100.0 / tc.total)::DOUBLE PRECISION
        ELSE 0::DOUBLE PRECISION
    END AS percentage
FROM agent_conversation_counts ac
CROSS JOIN total_conversations tc
ORDER BY ac.conversation_count DESC
