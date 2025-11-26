-- Optimized: Complete execution and update context timestamp in 1 query
-- Uses CTE to combine: UPDATE execution + SELECT context_id + UPDATE context
--
-- Parameters:
-- $1 = completed_at
-- $2 = output
-- $3 = output_schema
-- $4 = status
-- $5 = error_message
-- $6 = mcp_execution_id

WITH exec_update AS (
    UPDATE mcp_tool_executions
    SET
        completed_at = $1,
        output = $2,
        output_schema = $3,
        status = $4,
        error_message = $5,
        execution_time_ms = CAST(EXTRACT(EPOCH FROM ($1::timestamp - started_at)) * 1000 AS INTEGER)
    WHERE mcp_execution_id = $6 AND completed_at IS NULL
    RETURNING context_id
)
UPDATE user_contexts
SET updated_at = CURRENT_TIMESTAMP
WHERE context_id = (SELECT context_id FROM exec_update WHERE context_id IS NOT NULL AND context_id != '')
