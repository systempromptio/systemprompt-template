SELECT
    tool_name as label,
    COUNT(*) as value,
    ROUND(AVG(execution_time_ms) / 1000.0, 1) || 's avg' as badge,
    ROW_NUMBER() OVER (ORDER BY COUNT(*) DESC) as rank
FROM mcp_tool_executions
WHERE created_at >= datetime('now', '-' || $1 || ' days')
    AND status = 'success'
GROUP BY tool_name
ORDER BY value DESC
LIMIT $2;
