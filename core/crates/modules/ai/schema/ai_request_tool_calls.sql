-- ============================================================================
-- AI REQUEST TOOL CALLS - Normalized AI tool call tracking
-- ============================================================================
--
-- DATA RETENTION: Tool calls are automatically deleted via CASCADE when parent
-- ai_request is deleted. Follow ai_requests retention policy.
-- ============================================================================
CREATE TABLE IF NOT EXISTS ai_request_tool_calls (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id VARCHAR(255) NOT NULL,
    tool_name VARCHAR(255) NOT NULL,
    tool_input TEXT NOT NULL,
    mcp_execution_id VARCHAR(255),
    ai_tool_call_id VARCHAR(255),
    sequence_number INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (request_id) REFERENCES ai_requests(id) ON DELETE CASCADE,
    FOREIGN KEY (mcp_execution_id) REFERENCES mcp_tool_executions(mcp_execution_id) ON DELETE SET NULL,
    UNIQUE(request_id, sequence_number)
);
CREATE INDEX IF NOT EXISTS idx_ai_request_tool_calls_request_id ON ai_request_tool_calls(request_id);
CREATE INDEX IF NOT EXISTS idx_ai_request_tool_calls_tool_name ON ai_request_tool_calls(tool_name);
CREATE INDEX IF NOT EXISTS idx_ai_request_tool_calls_mcp_execution_id ON ai_request_tool_calls(mcp_execution_id);
CREATE INDEX IF NOT EXISTS idx_ai_request_tool_calls_ai_tool_call_id ON ai_request_tool_calls(ai_tool_call_id) WHERE ai_tool_call_id IS NOT NULL;
