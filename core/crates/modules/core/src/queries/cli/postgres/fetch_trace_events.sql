SELECT
    'LOG' as type,
    timestamp,
    level || ': ' || module || ' - ' || message as details,
    user_id,
    session_id,
    task_id,
    context_id,
    metadata
FROM logs
WHERE trace_id = $1

UNION ALL

SELECT
    'AI' as type,
    created_at as timestamp,
    provider || '/' || model || ' (' || status || ')' as details,
    user_id,
    session_id,
    task_id,
    context_id,
    jsonb_build_object(
        'request_id', request_id,
        'tokens_used', tokens_used,
        'cost_cents', cost_cents,
        'latency_ms', latency_ms,
        'error_message', error_message
    )::text as metadata
FROM ai_requests
WHERE trace_id = $2

UNION ALL

SELECT
    'TASK' as type,
    created_at as timestamp,
    'Task ' || substring(uuid, 1, 8) || '... (' || status || ')' as details,
    user_id,
    session_id,
    uuid as task_id,
    context_id,
    metadata
FROM agent_tasks
WHERE trace_id = $3

UNION ALL

SELECT
    'MESSAGE' as type,
    created_at as timestamp,
    role || ' message: ' || substring(message_id, 1, 8) || '...' as details,
    user_id,
    session_id,
    task_id,
    context_id,
    metadata
FROM task_messages
WHERE trace_id = $4

UNION ALL

SELECT
    'MCP' as type,
    started_at as timestamp,
    tool_name || ' on ' || mcp_server_name || ' (' || status || ')' as details,
    user_id,
    session_id,
    task_id,
    context_id,
    jsonb_build_object(
        'execution_time_ms', execution_time_ms,
        'error_message', error_message,
        'request_method', request_method,
        'request_source', request_source
    )::text as metadata
FROM mcp_tool_executions
WHERE trace_id = $5

ORDER BY timestamp ASC
