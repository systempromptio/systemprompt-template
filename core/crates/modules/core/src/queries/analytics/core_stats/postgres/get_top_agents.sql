SELECT
    t.agent_name as label,
    COUNT(*) as value,
    ROUND(
        CAST(SUM(CASE WHEN t.status = 'completed' THEN 1 ELSE 0 END) AS REAL)
        / COUNT(*) * 100,
        1
    ) || '% success' as badge,
    ROW_NUMBER() OVER (ORDER BY COUNT(*) DESC) as rank
FROM agent_tasks t
WHERE t.created_at >= datetime('now', '-' || $1 || ' days')
    AND t.agent_name IS NOT NULL
GROUP BY t.agent_name
ORDER BY value DESC
LIMIT $2;
