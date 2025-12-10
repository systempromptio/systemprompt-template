-- Add mcp_execution_id column to link AI requests to MCP tool executions
-- This enables tracing AI requests made by MCP tools back to their parent execution

ALTER TABLE ai_requests ADD COLUMN IF NOT EXISTS mcp_execution_id VARCHAR(255);

CREATE INDEX IF NOT EXISTS idx_ai_requests_mcp_execution_id ON ai_requests(mcp_execution_id);
