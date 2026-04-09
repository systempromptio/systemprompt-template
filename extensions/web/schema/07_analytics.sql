-- Consolidated schema: Analytics, usage aggregations, transcripts, governance

CREATE TABLE IF NOT EXISTS user_activity (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    category TEXT NOT NULL,
    action TEXT NOT NULL,
    entity_type TEXT,
    entity_id TEXT,
    entity_name TEXT,
    description TEXT NOT NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_user_activity_user ON user_activity(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_user_activity_category ON user_activity(category, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_user_activity_created ON user_activity(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_user_activity_mcp_access ON user_activity(category, created_at DESC) WHERE category = 'mcp_access';

CREATE TABLE IF NOT EXISTS plugin_usage_daily (
    id TEXT PRIMARY KEY,
    date DATE NOT NULL,
    plugin_id TEXT,
    event_type TEXT NOT NULL,
    tool_name TEXT,
    user_id TEXT NOT NULL,
    event_count BIGINT NOT NULL DEFAULT 0,
    total_duration_ms BIGINT DEFAULT 0,
    total_input_tokens BIGINT DEFAULT 0,
    total_output_tokens BIGINT DEFAULT 0,
    error_count BIGINT NOT NULL DEFAULT 0,
    content_input_bytes BIGINT DEFAULT 0,
    content_output_bytes BIGINT DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_usage_daily_unique ON plugin_usage_daily(date, user_id, event_type, COALESCE(tool_name, ''));
CREATE INDEX IF NOT EXISTS idx_usage_daily_date ON plugin_usage_daily(date DESC);
CREATE INDEX IF NOT EXISTS idx_usage_daily_plugin ON plugin_usage_daily(plugin_id, date DESC);
CREATE INDEX IF NOT EXISTS idx_usage_daily_user ON plugin_usage_daily(user_id, date DESC);

CREATE TABLE IF NOT EXISTS plugin_session_summaries (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL UNIQUE,
    user_id TEXT NOT NULL,
    plugin_id TEXT,
    started_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ,
    total_events BIGINT NOT NULL DEFAULT 0,
    tool_uses BIGINT NOT NULL DEFAULT 0,
    prompts BIGINT NOT NULL DEFAULT 0,
    errors BIGINT NOT NULL DEFAULT 0,
    total_input_tokens BIGINT DEFAULT 0,
    total_output_tokens BIGINT DEFAULT 0,
    model TEXT,
    status TEXT,
    unique_files_touched INT,
    content_input_bytes BIGINT NOT NULL DEFAULT 0,
    content_output_bytes BIGINT NOT NULL DEFAULT 0,
    ai_title TEXT,
    ai_summary TEXT,
    ai_tags TEXT,
    ai_description TEXT,
    apm REAL,
    eapm REAL,
    peak_concurrent INT,
    permission_mode TEXT,
    client_source TEXT,
    subagent_spawns BIGINT NOT NULL DEFAULT 0,
    user_prompts INT,
    automated_actions INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_session_summary_user ON plugin_session_summaries(user_id, started_at DESC);
CREATE INDEX IF NOT EXISTS idx_session_summary_session ON plugin_session_summaries(session_id);
CREATE INDEX IF NOT EXISTS idx_session_summary_source ON plugin_session_summaries(user_id, client_source);
CREATE INDEX IF NOT EXISTS idx_session_summary_mode ON plugin_session_summaries(user_id, permission_mode);

CREATE TABLE IF NOT EXISTS session_transcripts (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    plugin_id TEXT,
    transcript JSONB NOT NULL DEFAULT '[]',
    total_input_tokens BIGINT DEFAULT 0,
    total_output_tokens BIGINT DEFAULT 0,
    model TEXT,
    entries_counted INT DEFAULT 0,
    captured_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_session_transcripts_user ON session_transcripts(user_id, captured_at DESC);
CREATE INDEX IF NOT EXISTS idx_session_transcripts_session ON session_transcripts(session_id, captured_at DESC);

CREATE TABLE IF NOT EXISTS governance_decisions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    tool_name TEXT NOT NULL,
    agent_id TEXT,
    agent_scope TEXT,
    decision TEXT NOT NULL CHECK (decision IN ('allow', 'deny')),
    policy TEXT NOT NULL,
    reason TEXT NOT NULL,
    evaluated_rules JSONB DEFAULT '[]',
    plugin_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_governance_decisions_user ON governance_decisions(user_id);
CREATE INDEX IF NOT EXISTS idx_governance_decisions_session ON governance_decisions(session_id);
CREATE INDEX IF NOT EXISTS idx_governance_decisions_decision ON governance_decisions(decision);
CREATE INDEX IF NOT EXISTS idx_governance_decisions_created ON governance_decisions(created_at);
CREATE INDEX IF NOT EXISTS idx_governance_decisions_rate_limit ON governance_decisions(session_id, user_id, created_at DESC);
