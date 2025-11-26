SELECT tool_name, tool_input, mcp_execution_id
FROM ai_request_tool_calls
WHERE request_id = $1
ORDER BY sequence_number ASC
