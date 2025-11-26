-- Get new MCP tool executions since last seen timestamp
-- Used by SSE stream to send delta updates for ToolExecutionCompleted events
--
-- Parameters:
-- $1 = user_id
-- $2 = context_id
-- $3 = last_seen_timestamp

SELECT
    mte.id as execution_id,
    mte.context_id,
    mte.tool_name,
    mte.mcp_server_name as server_name,
    mte.output,
    mte.output_schema,
    mte.status,
    mte.created_at
FROM mcp_tool_executions mte
WHERE mte.user_id = $1
  AND mte.context_id = $2
  AND mte.completed_at > $3
  AND mte.status = 'success'
ORDER BY mte.completed_at ASC;
