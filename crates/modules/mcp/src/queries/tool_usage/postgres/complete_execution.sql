UPDATE mcp_tool_executions
SET
    completed_at = $1,
    output = $2,
    output_schema = $3,
    status = $4,
    error_message = $5,
    execution_time_ms = CAST(EXTRACT(EPOCH FROM ($1::timestamp - started_at)) * 1000 AS INTEGER)
WHERE mcp_execution_id = $6 AND completed_at IS NULL
