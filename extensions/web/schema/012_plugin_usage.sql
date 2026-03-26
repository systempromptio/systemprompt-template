CREATE TABLE IF NOT EXISTS plugin_usage_events (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    tool_name TEXT,
    plugin_id TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_plugin_usage_user ON plugin_usage_events(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_plugin_usage_session ON plugin_usage_events(session_id);
CREATE INDEX IF NOT EXISTS idx_plugin_usage_event_type ON plugin_usage_events(event_type);
CREATE INDEX IF NOT EXISTS idx_plugin_usage_tool_name ON plugin_usage_events(tool_name) WHERE tool_name IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_plugin_usage_created_at ON plugin_usage_events(created_at DESC);
