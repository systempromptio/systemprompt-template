-- Migration: Document mcp_execution_id field for tool call linkage
-- Purpose: Enable tracing of AI tool calls to their MCP executions
-- Status: Field already exists in schema, this migration documents the linkage mechanism

DO $$
BEGIN
    -- Verify column exists
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name='ai_request_tool_calls'
        AND column_name='mcp_execution_id'
    ) THEN
        RAISE EXCEPTION 'Column mcp_execution_id does not exist on ai_request_tool_calls table!';
    END IF;
END $$;

-- Update column comment to document the linkage
COMMENT ON COLUMN ai_request_tool_calls.mcp_execution_id IS
'Links this AI tool call request to an MCP tool execution.
Set after the tool execution completes successfully.
Enables tracing from AI requests through tool calls to actual MCP executions.
Foreign key to mcp_tool_executions.mcp_execution_id';

-- Verify the foreign key exists
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.table_constraints
        WHERE table_name='ai_request_tool_calls'
        AND constraint_name='ai_request_tool_calls_mcp_execution_id_fkey'
        AND constraint_type='FOREIGN KEY'
    ) THEN
        RAISE EXCEPTION 'Foreign key ai_request_tool_calls_mcp_execution_id_fkey does not exist!';
    END IF;
END $$;
