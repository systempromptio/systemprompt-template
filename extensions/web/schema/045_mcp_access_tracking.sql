-- Partial index for efficient MCP access event queries on the dashboard
CREATE INDEX IF NOT EXISTS idx_user_activity_mcp_access
  ON user_activity(category, created_at DESC)
  WHERE category = 'mcp_access';
