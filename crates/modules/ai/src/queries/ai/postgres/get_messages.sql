SELECT role, content, name, tool_call_id
FROM ai_request_messages
WHERE request_id = $1
ORDER BY sequence_number ASC
