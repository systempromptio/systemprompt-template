-- Migration 047: Admin dashboard views — consolidated table creation.
-- All statements use IF NOT EXISTS so they are safe to re-run.

-- ============================================================
-- Marketplace schema
-- ============================================================
CREATE SCHEMA IF NOT EXISTS marketplace;

-- ============================================================
-- 1. tenant_activity (acquisition metrics, sparkline signups)
-- ============================================================
CREATE TABLE IF NOT EXISTS tenant_activity (
    id BIGSERIAL PRIMARY KEY,
    user_id TEXT,
    event_type TEXT NOT NULL,
    remote_created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_tenant_activity_event ON tenant_activity(event_type);
CREATE INDEX IF NOT EXISTS idx_tenant_activity_remote ON tenant_activity(remote_created_at DESC);
CREATE INDEX IF NOT EXISTS idx_tenant_activity_user ON tenant_activity(user_id);

-- ============================================================
-- 2. admin_traffic_reports (daily report snapshots)
-- ============================================================
CREATE TABLE IF NOT EXISTS admin_traffic_reports (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    report_date DATE NOT NULL,
    report_period TEXT NOT NULL,
    report_data JSONB NOT NULL DEFAULT '{}',
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (report_date, report_period)
);

CREATE INDEX IF NOT EXISTS idx_atr_date ON admin_traffic_reports(report_date DESC, generated_at DESC);

-- ============================================================
-- 3. daily_summaries (per-user daily roll-up)
-- ============================================================
CREATE TABLE IF NOT EXISTS daily_summaries (
    user_id TEXT NOT NULL,
    summary_date DATE NOT NULL,
    session_count INT NOT NULL DEFAULT 0,
    avg_quality_score REAL,
    goals_achieved INT NOT NULL DEFAULT 0,
    goals_partial INT NOT NULL DEFAULT 0,
    goals_failed INT NOT NULL DEFAULT 0,
    total_prompts BIGINT NOT NULL DEFAULT 0,
    total_tool_uses BIGINT NOT NULL DEFAULT 0,
    total_errors BIGINT NOT NULL DEFAULT 0,
    summary TEXT NOT NULL DEFAULT '',
    patterns TEXT,
    skill_gaps TEXT,
    top_recommendation TEXT,
    daily_xp INT NOT NULL DEFAULT 0,
    tags TEXT NOT NULL DEFAULT '',
    avg_apm REAL,
    peak_apm REAL,
    avg_eapm REAL,
    peak_concurrency INT NOT NULL DEFAULT 0,
    avg_concurrency REAL,
    total_input_bytes BIGINT NOT NULL DEFAULT 0,
    total_output_bytes BIGINT NOT NULL DEFAULT 0,
    peak_throughput_bps BIGINT NOT NULL DEFAULT 0,
    tool_diversity INT NOT NULL DEFAULT 0,
    multitasking_score REAL,
    session_velocity REAL,
    achievements_unlocked TEXT NOT NULL DEFAULT '',
    highlights TEXT,
    trends TEXT,
    category_distribution JSONB,
    plugins_count INT NOT NULL DEFAULT 0,
    skills_count INT NOT NULL DEFAULT 0,
    agents_count INT NOT NULL DEFAULT 0,
    mcp_servers_count INT NOT NULL DEFAULT 0,
    hooks_count INT NOT NULL DEFAULT 0,
    health_score REAL,
    skill_effectiveness JSONB,
    avg_session_duration_minutes REAL,
    avg_turns_per_session REAL,
    total_corrections INT NOT NULL DEFAULT 0,
    avg_automation_ratio REAL,
    plan_mode_sessions INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, summary_date)
);

CREATE INDEX IF NOT EXISTS idx_daily_summaries_user ON daily_summaries(user_id, summary_date DESC);
CREATE INDEX IF NOT EXISTS idx_daily_summaries_date ON daily_summaries(summary_date DESC);

-- ============================================================
-- 4. session_analyses (AI session analysis results)
-- ============================================================
CREATE TABLE IF NOT EXISTS session_analyses (
    session_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    title TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    summary TEXT NOT NULL DEFAULT '',
    tags TEXT NOT NULL DEFAULT '',
    goal_achieved TEXT NOT NULL DEFAULT '',
    quality_score SMALLINT NOT NULL DEFAULT 0,
    outcome TEXT NOT NULL DEFAULT '',
    error_analysis TEXT,
    skill_assessment TEXT,
    recommendations TEXT,
    skill_scores JSONB,
    category TEXT NOT NULL DEFAULT 'other',
    goal_outcome_map JSONB,
    efficiency_metrics JSONB,
    best_practices_checklist JSONB,
    improvement_hints TEXT,
    corrections_count INT NOT NULL DEFAULT 0,
    session_duration_minutes INT,
    total_turns INT,
    automation_ratio REAL,
    plan_mode_used BOOLEAN NOT NULL DEFAULT false,
    client_surface TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_session_analyses_user ON session_analyses(user_id, created_at DESC);

-- ============================================================
-- 5. session_entity_links (entity usage per session)
-- ============================================================
CREATE TABLE IF NOT EXISTS session_entity_links (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    user_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_name TEXT NOT NULL,
    entity_id TEXT,
    usage_count INT NOT NULL DEFAULT 1,
    first_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, session_id, entity_type, entity_name)
);

CREATE INDEX IF NOT EXISTS idx_sel_user ON session_entity_links(user_id);
CREATE INDEX IF NOT EXISTS idx_sel_session ON session_entity_links(session_id);
CREATE INDEX IF NOT EXISTS idx_sel_entity ON session_entity_links(entity_type, entity_name);

-- ============================================================
-- 6. session_ratings (user feedback on sessions)
-- ============================================================
CREATE TABLE IF NOT EXISTS session_ratings (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    user_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    rating SMALLINT NOT NULL,
    outcome TEXT NOT NULL DEFAULT '',
    notes TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, session_id)
);

CREATE INDEX IF NOT EXISTS idx_sr_user ON session_ratings(user_id);
CREATE INDEX IF NOT EXISTS idx_sr_session ON session_ratings(session_id);

-- ============================================================
-- 7. user_achievements (unlocked achievements)
-- ============================================================
CREATE TABLE IF NOT EXISTS user_achievements (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    user_id TEXT NOT NULL,
    achievement_id TEXT NOT NULL,
    unlocked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, achievement_id)
);

CREATE INDEX IF NOT EXISTS idx_ua_user ON user_achievements(user_id);
CREATE INDEX IF NOT EXISTS idx_ua_unlocked ON user_achievements(unlocked_at DESC);

-- ============================================================
-- 8. user_ranks (leaderboard ranks)
-- ============================================================
CREATE TABLE IF NOT EXISTS user_ranks (
    user_id TEXT PRIMARY KEY,
    rank_name TEXT NOT NULL DEFAULT 'Spark',
    total_xp BIGINT NOT NULL DEFAULT 0,
    rank_level INT NOT NULL DEFAULT 1,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================
-- 9. skill_ratings (user feedback on skills)
-- ============================================================
CREATE TABLE IF NOT EXISTS skill_ratings (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    user_id TEXT NOT NULL,
    skill_name TEXT NOT NULL,
    rating SMALLINT NOT NULL,
    notes TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, skill_name)
);

CREATE INDEX IF NOT EXISTS idx_skr_user ON skill_ratings(user_id);

-- ============================================================
-- 10. user_settings (preferences and notification settings)
-- ============================================================
CREATE TABLE IF NOT EXISTS user_settings (
    user_id TEXT PRIMARY KEY,
    display_name TEXT,
    avatar_url TEXT,
    notify_daily_summary BOOLEAN NOT NULL DEFAULT true,
    notify_achievements BOOLEAN NOT NULL DEFAULT true,
    leaderboard_opt_in BOOLEAN NOT NULL DEFAULT false,
    timezone TEXT NOT NULL DEFAULT 'UTC',
    achievement_email_sent_date DATE,
    daily_report_email_sent_date DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================
-- 11. marketplace.plans (subscription plan definitions)
-- ============================================================
CREATE TABLE IF NOT EXISTS marketplace.plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT,
    paddle_product_id TEXT NOT NULL,
    paddle_price_id TEXT NOT NULL,
    amount_cents INT NOT NULL DEFAULT 0,
    currency TEXT NOT NULL DEFAULT 'USD',
    billing_interval TEXT NOT NULL DEFAULT 'month',
    features JSONB NOT NULL DEFAULT '{}',
    limits JSONB NOT NULL DEFAULT '{}',
    sort_order INT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================
-- 12. marketplace.paddle_customers (Paddle customer mapping)
-- ============================================================
CREATE TABLE IF NOT EXISTS marketplace.paddle_customers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id TEXT NOT NULL UNIQUE,
    paddle_customer_id TEXT UNIQUE,
    email TEXT NOT NULL,
    name TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_mpc_user ON marketplace.paddle_customers(user_id);
CREATE INDEX IF NOT EXISTS idx_mpc_paddle ON marketplace.paddle_customers(paddle_customer_id);

-- ============================================================
-- 13. marketplace.subscriptions (user subscriptions)
-- ============================================================
CREATE TABLE IF NOT EXISTS marketplace.subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id TEXT NOT NULL,
    paddle_subscription_id TEXT UNIQUE,
    paddle_customer_id TEXT,
    plan_id UUID REFERENCES marketplace.plans(id),
    status TEXT NOT NULL DEFAULT 'active',
    current_period_start TIMESTAMPTZ,
    current_period_end TIMESTAMPTZ,
    cancel_at TIMESTAMPTZ,
    paddle_data JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ms_user ON marketplace.subscriptions(user_id);
CREATE INDEX IF NOT EXISTS idx_ms_paddle ON marketplace.subscriptions(paddle_subscription_id);
CREATE INDEX IF NOT EXISTS idx_ms_status ON marketplace.subscriptions(status);

-- ============================================================
-- 14. marketplace.paddle_webhook_events (idempotent webhook log)
-- ============================================================
CREATE TABLE IF NOT EXISTS marketplace.paddle_webhook_events (
    id BIGSERIAL PRIMARY KEY,
    event_id TEXT NOT NULL UNIQUE,
    event_type TEXT NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'pending',
    error_message TEXT,
    processed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_mpwe_event ON marketplace.paddle_webhook_events(event_id);
CREATE INDEX IF NOT EXISTS idx_mpwe_type ON marketplace.paddle_webhook_events(event_type);

-- ============================================================
-- 15. Missing column on plugin_session_summaries: model
--     Referenced by control_center/sessions.rs and
--     usage_aggregations/session_updates.rs but not added
--     in earlier migrations.
-- ============================================================
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS model TEXT;
