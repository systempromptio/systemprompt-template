SELECT
    mcp_execution_id,
    tool_name,
    mcp_server_name,
    input,
    output,
    status
FROM mcp_tool_executions
WHERE mcp_execution_id = $1
