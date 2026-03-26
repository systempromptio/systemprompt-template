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
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_usage_daily_unique
    ON plugin_usage_daily(date, user_id, event_type, COALESCE(plugin_id, ''), COALESCE(tool_name, ''));
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
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_session_summary_user ON plugin_session_summaries(user_id, started_at DESC);
CREATE INDEX IF NOT EXISTS idx_session_summary_session ON plugin_session_summaries(session_id);
