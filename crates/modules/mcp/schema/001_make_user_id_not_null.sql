-- Migration: Make user_id NOT NULL on mcp_tool_executions
-- MCP tool executions must be attributed to users for analytics
BEGIN;

ALTER TABLE mcp_tool_executions
ALTER COLUMN user_id SET NOT NULL;

COMMIT;
