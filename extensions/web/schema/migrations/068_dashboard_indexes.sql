CREATE INDEX IF NOT EXISTS idx_plugin_usage_session_created ON plugin_usage_events(session_id, created_at);
CREATE INDEX IF NOT EXISTS idx_governance_decisions_policy_created ON governance_decisions(policy, created_at);
CREATE INDEX IF NOT EXISTS idx_governance_decisions_tool_name ON governance_decisions(tool_name);
