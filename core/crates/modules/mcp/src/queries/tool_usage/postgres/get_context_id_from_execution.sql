-- Get context_id from tool execution record
-- Used to trigger SSE updates when tool execution completes
--
-- Parameters:
-- $1 = mcp_execution_id

SELECT context_id
FROM mcp_tool_executions
WHERE mcp_execution_id = $1
