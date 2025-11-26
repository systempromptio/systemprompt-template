SELECT
    tool_name,
    mcp_server_name,
    COUNT(*) as call_count,
    AVG(execution_time_ms) as avg_duration_ms,
    MIN(execution_time_ms) as min_duration_ms,
    MAX(execution_time_ms) as max_duration_ms,
    SUM(CASE WHEN status = 'success' THEN 1 ELSE 0 END) * 100.0 / COUNT(*) as success_rate,
    COUNT(DISTINCT session_id) as unique_sessions,
    COUNT(DISTINCT COALESCE(user_id, session_id)) as unique_users,
    COUNT(DISTINCT context_id) as unique_contexts
FROM mcp_tool_executions
WHERE started_at >= NOW() - ($1 || ' days')::INTERVAL
