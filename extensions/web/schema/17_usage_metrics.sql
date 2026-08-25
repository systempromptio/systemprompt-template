-- Usage metrics captured from Claude Code hook telemetry and the gateway,
-- feeding the admin analytics dashboard.
--
-- These are web-owned tables. They reference core-owned users only by TEXT id
-- (the plugin_usage_events precedent) — no FK onto core tables.

-- One row per git commit observed through a Claude Code Bash tool call.
-- Only commits made inside tracked sessions are visible; amends and rebases
-- mint new hashes and count as new commits. Stats columns are NULL when the
-- "N files changed, X insertions, Y deletions" stdout line was absent.
CREATE TABLE IF NOT EXISTS user_commits (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    user_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    cwd TEXT,
    branch TEXT,
    commit_hash TEXT NOT NULL,
    message TEXT NOT NULL DEFAULT '',
    files_changed INT,
    insertions INT,
    deletions INT,
    committed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
-- Retries and duplicate hook deliveries collapse here: the same commit in the
-- same repo by the same user is one row.
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_commits_dedup
    ON user_commits(user_id, COALESCE(cwd, ''), commit_hash);
CREATE INDEX IF NOT EXISTS idx_user_commits_user
    ON user_commits(user_id, committed_at DESC);
CREATE INDEX IF NOT EXISTS idx_user_commits_day
    ON user_commits(committed_at);

-- Latest statusline snapshot per session: cumulative client-reported cost and
-- context-window token usage. Set-not-increment semantics — the client reports
-- running totals, so each upsert replaces the previous snapshot.
CREATE TABLE IF NOT EXISTS session_cost_snapshots (
    session_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    model TEXT,
    total_cost_microdollars BIGINT,
    context_window_size BIGINT,
    input_tokens BIGINT,
    output_tokens BIGINT,
    cache_creation_input_tokens BIGINT,
    cache_read_input_tokens BIGINT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_session_cost_snapshots_user
    ON session_cost_snapshots(user_id, updated_at DESC);

-- Per-user-per-day rollup joining the hook plane (sessions, prompts, LOC),
-- observed commits, and the gateway plane (ai_requests tokens + cost), so the
-- dashboard reads one narrow table instead of re-aggregating raw events.
-- Recomputed idempotently by the usage_daily_rollup job for a trailing window.
CREATE TABLE IF NOT EXISTS admin_usage_daily_rollups (
    user_id TEXT NOT NULL,
    date DATE NOT NULL,
    sessions_count INT NOT NULL DEFAULT 0,
    prompts BIGINT NOT NULL DEFAULT 0,
    tool_uses BIGINT NOT NULL DEFAULT 0,
    errors BIGINT NOT NULL DEFAULT 0,
    loc_added_ai BIGINT NOT NULL DEFAULT 0,
    loc_removed_ai BIGINT NOT NULL DEFAULT 0,
    commits_count INT NOT NULL DEFAULT 0,
    -- Whole-diff line counts from git stdout: AI and manual lines together,
    -- a different measurement frame from loc_added_ai. The dashboard shows
    -- both and never subtracts one from the other.
    commit_insertions BIGINT NOT NULL DEFAULT 0,
    commit_deletions BIGINT NOT NULL DEFAULT 0,
    ai_requests_count BIGINT NOT NULL DEFAULT 0,
    input_tokens BIGINT NOT NULL DEFAULT 0,
    output_tokens BIGINT NOT NULL DEFAULT 0,
    cost_microdollars BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, date)
);
CREATE INDEX IF NOT EXISTS idx_admin_usage_rollups_date
    ON admin_usage_daily_rollups(date DESC);

-- One row per organization per calendar month per kind, recording a budget
-- threshold event: 'soft_cap' when month-to-date spend crossed the plan's
-- warning threshold, 'forecast_overrun' when the linear month-end projection
-- first exceeded the hard cap. Upserted (never denied on) by the gateway
-- budget guard; read by the dashboard's spend view.
CREATE TABLE IF NOT EXISTS org_budget_warnings (
    org_id TEXT NOT NULL,
    month DATE NOT NULL,
    kind TEXT NOT NULL DEFAULT 'soft_cap'
        CHECK (kind IN ('soft_cap', 'forecast_overrun')),
    threshold_microdollars BIGINT NOT NULL,
    spent_microdollars BIGINT NOT NULL,
    first_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (org_id, month, kind)
);

-- Persisted usage anomalies, one row per metric per hourly window (see
-- migrations/032_usage_anomalies.sql for the rationale). Written by the
-- usage_anomaly job, read by the spend dashboard.
CREATE TABLE IF NOT EXISTS usage_anomalies (
    metric TEXT NOT NULL CHECK (metric IN ('requests', 'cost', 'errors')),
    window_start TIMESTAMPTZ NOT NULL,
    observed BIGINT NOT NULL,
    baseline BIGINT NOT NULL,
    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (metric, window_start)
);

CREATE INDEX IF NOT EXISTS idx_usage_anomalies_detected
    ON usage_anomalies(detected_at DESC);
