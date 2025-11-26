INSERT INTO mcp_tool_executions (
    mcp_execution_id,
    tool_name,
    mcp_server_name,
    started_at,
    completed_at,
    execution_time_ms,
    input,
    output,
    output_schema,
    status,
    error_message,
    user_id,
    session_id,
    context_id,
    task_id,
    trace_id,
    request_method,
    request_source,
    ai_tool_call_id
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
