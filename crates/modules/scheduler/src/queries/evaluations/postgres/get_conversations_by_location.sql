WITH total AS (
    SELECT COUNT(DISTINCT ce.context_id) as total_count
    FROM conversation_evaluations ce
    INNER JOIN agent_tasks at ON ce.context_id = at.context_id
    INNER JOIN user_sessions s ON at.session_id = s.session_id
    WHERE ce.analyzed_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
        AND s.country IS NOT NULL
)
SELECT
    COALESCE(s.country, 'Unknown') as country,
    COUNT(DISTINCT ce.context_id) as conversation_count,
    ROUND((COUNT(DISTINCT ce.context_id) * 100.0 / total.total_count), 1) as percentage,
    COALESCE(AVG(ce.overall_score), 0.0) as avg_quality
FROM conversation_evaluations ce
INNER JOIN agent_tasks at ON ce.context_id = at.context_id
INNER JOIN user_sessions s ON at.session_id = s.session_id
CROSS JOIN total
WHERE ce.analyzed_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
    AND s.country IS NOT NULL
GROUP BY s.country, total.total_count
ORDER BY conversation_count DESC
LIMIT $2
