DELETE FROM mcp_tool_executions mcp
WHERE mcp.session_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1 FROM user_sessions us
    WHERE us.session_id = mcp.session_id
  )
