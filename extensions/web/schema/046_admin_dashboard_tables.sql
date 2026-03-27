-- Migration 046: Admin dashboard tables and columns
-- Ported from systemprompt-web marketplace + web extensions
-- All statements are idempotent (IF NOT EXISTS / ADD COLUMN IF NOT EXISTS)

-- ============================================================
-- 1. Missing columns on plugin_usage_events
--    (upstream: marketplace/003_plugin_usage.sql)
-- ============================================================
ALTER TABLE plugin_usage_events ADD COLUMN IF NOT EXISTS prompt_preview TEXT NOT NULL DEFAULT '';
ALTER TABLE plugin_usage_events ADD COLUMN IF NOT EXISTS description TEXT NOT NULL DEFAULT '';
ALTER TABLE plugin_usage_events ADD COLUMN IF NOT EXISTS cwd TEXT NOT NULL DEFAULT '';
ALTER TABLE plugin_usage_events ADD COLUMN IF NOT EXISTS dedup_key TEXT;
ALTER TABLE plugin_usage_events ADD COLUMN IF NOT EXISTS content_input_bytes BIGINT DEFAULT 0;
ALTER TABLE plugin_usage_events ADD COLUMN IF NOT EXISTS content_output_bytes BIGINT DEFAULT 0;

CREATE UNIQUE INDEX IF NOT EXISTS idx_plugin_usage_dedup ON plugin_usage_events(dedup_key) WHERE dedup_key IS NOT NULL;

-- ============================================================
-- 2. Missing columns on plugin_session_summaries
--    (upstream: marketplace/006_usage_aggregations.sql)
-- ============================================================
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT 'active';
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS unique_files_touched INT DEFAULT 0;
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS content_input_bytes BIGINT DEFAULT 0;
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS content_output_bytes BIGINT DEFAULT 0;
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS ai_title TEXT;
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS ai_summary TEXT;
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS ai_tags TEXT;
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS ai_description TEXT;
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS apm REAL;
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS eapm REAL;
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS peak_concurrent INT DEFAULT 0;
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS permission_mode TEXT NOT NULL DEFAULT '';
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS client_source TEXT NOT NULL DEFAULT '';
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS subagent_spawns INT DEFAULT 0;
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS user_prompts INT NOT NULL DEFAULT 0;
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS automated_actions INT NOT NULL DEFAULT 0;
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS model TEXT NOT NULL DEFAULT '';

CREATE INDEX IF NOT EXISTS idx_session_summary_source ON plugin_session_summaries(user_id, client_source);
CREATE INDEX IF NOT EXISTS idx_session_summary_mode ON plugin_session_summaries(user_id, permission_mode);

-- ============================================================
-- 3. Missing columns on plugin_usage_daily
--    (upstream: marketplace/006_usage_aggregations.sql)
-- ============================================================
ALTER TABLE plugin_usage_daily ADD COLUMN IF NOT EXISTS content_input_bytes BIGINT DEFAULT 0;
ALTER TABLE plugin_usage_daily ADD COLUMN IF NOT EXISTS content_output_bytes BIGINT DEFAULT 0;

-- ============================================================
-- 4. Missing column on user_hooks
--    (upstream: marketplace/028_user_hooks.sql)
-- ============================================================
ALTER TABLE user_hooks ADD COLUMN IF NOT EXISTS plugin_id TEXT;
ALTER TABLE user_hooks ADD COLUMN IF NOT EXISTS is_default BOOLEAN NOT NULL DEFAULT false;
CREATE INDEX IF NOT EXISTS idx_user_hooks_plugin ON user_hooks(plugin_id);

-- ============================================================
-- 5. tenant_activity
--    (upstream: web/012_tenant_activity.sql)
-- ============================================================
CREATE TABLE IF NOT EXISTS tenant_activity (
    id SERIAL PRIMARY KEY,
    external_id TEXT UNIQUE NOT NULL,
    tenant_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    user_id TEXT,
    user_email TEXT,
    event_source TEXT,
    event_data JSONB,
    remote_created_at TIMESTAMPTZ NOT NULL,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_tenant_activity_tenant_id ON tenant_activity(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_activity_event_type ON tenant_activity(event_type);
CREATE INDEX IF NOT EXISTS idx_tenant_activity_remote_created_at ON tenant_activity(remote_created_at DESC);
CREATE INDEX IF NOT EXISTS idx_tenant_activity_sync ON tenant_activity(remote_created_at, synced_at);

-- ============================================================
-- 6. admin_traffic_reports
--    (upstream: marketplace/049_admin_traffic_reports.sql)
-- ============================================================
CREATE TABLE IF NOT EXISTS admin_traffic_reports (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    report_date DATE NOT NULL,
    report_period TEXT NOT NULL DEFAULT 'am',
    report_data JSONB NOT NULL DEFAULT '{}'::jsonb,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (report_date, report_period)
);
CREATE INDEX IF NOT EXISTS idx_admin_traffic_reports_date ON admin_traffic_reports(report_date DESC);

-- ============================================================
-- 7. session_entity_links
--    (upstream: marketplace/032_conversation_analytics.sql)
-- ============================================================
CREATE TABLE IF NOT EXISTS session_entity_links (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    user_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    entity_type TEXT NOT NULL CHECK (entity_type IN ('skill', 'agent', 'mcp_tool')),
    entity_name TEXT NOT NULL,
    entity_id TEXT,
    usage_count INT NOT NULL DEFAULT 1,
    first_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, session_id, entity_type, entity_name)
);
CREATE INDEX IF NOT EXISTS idx_session_entity_user ON session_entity_links(user_id);
CREATE INDEX IF NOT EXISTS idx_session_entity_session ON session_entity_links(session_id);
CREATE INDEX IF NOT EXISTS idx_session_entity_type_name ON session_entity_links(entity_type, entity_name);

-- ============================================================
-- 8. session_ratings
--    (upstream: marketplace/032_conversation_analytics.sql)
-- ============================================================
CREATE TABLE IF NOT EXISTS session_ratings (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    user_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    rating SMALLINT NOT NULL CHECK (rating >= 1 AND rating <= 5),
    outcome TEXT NOT NULL DEFAULT '',
    notes TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, session_id)
);
CREATE INDEX IF NOT EXISTS idx_session_ratings_user ON session_ratings(user_id);

-- ============================================================
-- 9. skill_ratings
--    (upstream: marketplace/032_conversation_analytics.sql)
-- ============================================================
CREATE TABLE IF NOT EXISTS skill_ratings (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    user_id TEXT NOT NULL,
    skill_name TEXT NOT NULL,
    rating SMALLINT NOT NULL CHECK (rating >= 1 AND rating <= 5),
    notes TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, skill_name)
);
CREATE INDEX IF NOT EXISTS idx_skill_ratings_user ON skill_ratings(user_id);

-- ============================================================
-- 10. session_analyses
--     (upstream: marketplace/039_session_analyses.sql)
-- ============================================================
CREATE TABLE IF NOT EXISTS session_analyses (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    session_id TEXT NOT NULL UNIQUE,
    user_id TEXT NOT NULL,
    title TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    summary TEXT NOT NULL DEFAULT '',
    tags TEXT NOT NULL DEFAULT '',
    goal_achieved TEXT NOT NULL DEFAULT 'unknown'
        CHECK (goal_achieved IN ('yes', 'partial', 'no', 'unknown')),
    quality_score SMALLINT NOT NULL DEFAULT 3 CHECK (quality_score >= 1 AND quality_score <= 5),
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
    corrections_count INTEGER NOT NULL DEFAULT 0,
    session_duration_minutes INTEGER,
    total_turns INTEGER,
    automation_ratio REAL,
    plan_mode_used BOOLEAN DEFAULT false,
    client_surface TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_session_analyses_user ON session_analyses(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_session_analyses_session ON session_analyses(session_id);
CREATE INDEX IF NOT EXISTS idx_session_analyses_quality ON session_analyses(user_id, quality_score);
CREATE INDEX IF NOT EXISTS idx_session_analyses_category ON session_analyses(user_id, category);

-- ============================================================
-- 11. daily_summaries
--     (upstream: marketplace/040_daily_summaries.sql)
-- ============================================================
CREATE TABLE IF NOT EXISTS daily_summaries (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    user_id TEXT NOT NULL,
    summary_date DATE NOT NULL,
    session_count INTEGER NOT NULL DEFAULT 0,
    avg_quality_score REAL,
    goals_achieved INTEGER NOT NULL DEFAULT 0,
    goals_partial INTEGER NOT NULL DEFAULT 0,
    goals_failed INTEGER NOT NULL DEFAULT 0,
    total_prompts BIGINT NOT NULL DEFAULT 0,
    total_tool_uses BIGINT NOT NULL DEFAULT 0,
    total_errors BIGINT NOT NULL DEFAULT 0,
    summary TEXT NOT NULL DEFAULT '',
    patterns TEXT,
    skill_gaps TEXT,
    top_recommendation TEXT,
    daily_xp INTEGER NOT NULL DEFAULT 0,
    tags TEXT NOT NULL DEFAULT '',
    avg_apm REAL,
    peak_apm REAL,
    avg_eapm REAL,
    peak_concurrency INT DEFAULT 0,
    avg_concurrency REAL,
    total_input_bytes BIGINT DEFAULT 0,
    total_output_bytes BIGINT DEFAULT 0,
    peak_throughput_bps BIGINT DEFAULT 0,
    tool_diversity INT DEFAULT 0,
    multitasking_score REAL,
    session_velocity REAL,
    achievements_unlocked TEXT DEFAULT '',
    highlights TEXT,
    trends TEXT,
    category_distribution JSONB DEFAULT '{}'::jsonb,
    plugins_count INTEGER NOT NULL DEFAULT 0,
    skills_count INTEGER NOT NULL DEFAULT 0,
    agents_count INTEGER NOT NULL DEFAULT 0,
    mcp_servers_count INTEGER NOT NULL DEFAULT 0,
    hooks_count INTEGER NOT NULL DEFAULT 0,
    health_score REAL,
    skill_effectiveness JSONB DEFAULT '[]'::jsonb,
    avg_session_duration_minutes REAL,
    avg_turns_per_session REAL,
    total_corrections INTEGER DEFAULT 0,
    avg_automation_ratio REAL,
    plan_mode_sessions INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, summary_date)
);
CREATE INDEX IF NOT EXISTS idx_daily_summaries_user ON daily_summaries(user_id, summary_date DESC);

-- ============================================================
-- 12. user_settings
--     (upstream: marketplace/041_user_settings.sql)
-- ============================================================
CREATE TABLE IF NOT EXISTS user_settings (
    user_id TEXT PRIMARY KEY,
    display_name TEXT,
    avatar_url TEXT,
    notify_daily_summary BOOLEAN NOT NULL DEFAULT true,
    notify_achievements BOOLEAN NOT NULL DEFAULT true,
    leaderboard_opt_in BOOLEAN NOT NULL DEFAULT true,
    timezone TEXT NOT NULL DEFAULT 'UTC',
    achievement_email_sent_date DATE,
    daily_report_email_sent_date DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================
-- 13. Gamification tables
--     (upstream: marketplace/019_gamification.sql)
-- ============================================================
CREATE TABLE IF NOT EXISTS user_ranks (
    user_id TEXT PRIMARY KEY,
    total_xp INTEGER NOT NULL DEFAULT 0,
    rank_level INTEGER NOT NULL DEFAULT 1,
    rank_name TEXT NOT NULL DEFAULT 'Spark',
    events_count BIGINT NOT NULL DEFAULT 0,
    unique_skills_count INTEGER NOT NULL DEFAULT 0,
    unique_plugins_count INTEGER NOT NULL DEFAULT 0,
    current_streak INTEGER NOT NULL DEFAULT 0,
    longest_streak INTEGER NOT NULL DEFAULT 0,
    last_active_date DATE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_achievements (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    user_id TEXT NOT NULL,
    achievement_id TEXT NOT NULL,
    unlocked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, achievement_id)
);
CREATE INDEX IF NOT EXISTS idx_user_achievements_user ON user_achievements(user_id);
CREATE INDEX IF NOT EXISTS idx_user_achievements_id ON user_achievements(achievement_id);

-- ============================================================
-- 14. Marketplace schema (Paddle billing)
--     (upstream: marketplace/023_subscriptions.sql + 050_plan_roles.sql)
-- ============================================================
CREATE SCHEMA IF NOT EXISTS marketplace;

CREATE TABLE IF NOT EXISTS marketplace.plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT,
    paddle_product_id TEXT NOT NULL,
    paddle_price_id TEXT NOT NULL UNIQUE,
    amount_cents INTEGER NOT NULL,
    currency TEXT NOT NULL DEFAULT 'USD',
    billing_interval TEXT NOT NULL DEFAULT 'month',
    features JSONB NOT NULL DEFAULT '[]',
    limits JSONB NOT NULL DEFAULT '{}',
    role_name TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_plans_role_name ON marketplace.plans(role_name) WHERE role_name IS NOT NULL;

CREATE TABLE IF NOT EXISTS marketplace.paddle_customers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id TEXT NOT NULL UNIQUE,
    paddle_customer_id TEXT UNIQUE,
    email TEXT NOT NULL,
    name TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS marketplace.subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id TEXT NOT NULL,
    paddle_subscription_id TEXT UNIQUE,
    paddle_customer_id TEXT,
    plan_id UUID REFERENCES marketplace.plans(id),
    status TEXT NOT NULL DEFAULT 'pending',
    current_period_start TIMESTAMPTZ,
    current_period_end TIMESTAMPTZ,
    cancel_at TIMESTAMPTZ,
    paddle_data JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_subscriptions_user_id ON marketplace.subscriptions(user_id);
CREATE INDEX IF NOT EXISTS idx_subscriptions_paddle_sub_id ON marketplace.subscriptions(paddle_subscription_id);
CREATE INDEX IF NOT EXISTS idx_subscriptions_status ON marketplace.subscriptions(status);

CREATE TABLE IF NOT EXISTS marketplace.paddle_webhook_events (
    id BIGSERIAL PRIMARY KEY,
    event_id TEXT NOT NULL UNIQUE,
    event_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'received',
    payload JSONB,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    processed_at TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_webhook_events_event_id ON marketplace.paddle_webhook_events(event_id);

-- Seed default plans
INSERT INTO marketplace.plans (name, display_name, description, paddle_product_id, paddle_price_id, amount_cents, role_name, sort_order, limits)
VALUES (
    'free', 'Free', 'Default free tier', 'none', 'free_default', 0, NULL, 0,
    '{"ingestion":{"events_per_day":1000000,"content_bytes_per_day":10737418240,"sessions_per_day":10000},"entities":{"max_skills":1000,"max_agents":1000,"max_plugins":1000,"max_mcp_servers":1000,"max_hooks":1000},"features":{"ai_session_analysis":true,"ai_daily_summaries":true,"apm_metrics":true,"gamification":true,"export_zip":true},"api":{"requests_per_minute":6000}}'::jsonb
)
ON CONFLICT (name) DO UPDATE SET
    limits = EXCLUDED.limits,
    display_name = EXCLUDED.display_name;

INSERT INTO marketplace.plans (name, display_name, description, paddle_product_id, paddle_price_id, amount_cents, role_name, sort_order, limits)
VALUES (
    'admin', 'Admin', 'Administrator unlimited access', 'none', 'admin_role', 0, 'admin', 99,
    '{"ingestion":{"events_per_day":9223372036854775807,"content_bytes_per_day":9223372036854775807,"sessions_per_day":9223372036854775807},"entities":{"max_skills":9223372036854775807,"max_agents":9223372036854775807,"max_plugins":9223372036854775807,"max_mcp_servers":9223372036854775807,"max_hooks":9223372036854775807},"features":{"ai_session_analysis":true,"ai_daily_summaries":true,"apm_metrics":true,"gamification":true,"export_zip":true},"api":{"requests_per_minute":9223372036854775807}}'::jsonb
)
ON CONFLICT (name) DO UPDATE SET
    limits = EXCLUDED.limits,
    role_name = EXCLUDED.role_name,
    display_name = EXCLUDED.display_name;

-- ============================================================
-- 15. org_marketplace_sync_logs
-- ============================================================
CREATE TABLE IF NOT EXISTS org_marketplace_sync_logs (
    id BIGSERIAL PRIMARY KEY,
    marketplace_id TEXT NOT NULL,
    operation TEXT NOT NULL,
    status TEXT NOT NULL,
    commit_hash TEXT,
    plugins_synced BIGINT NOT NULL DEFAULT 0,
    errors BIGINT NOT NULL DEFAULT 0,
    error_message TEXT,
    triggered_by TEXT,
    duration_ms BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_omsl_marketplace ON org_marketplace_sync_logs(marketplace_id);
CREATE INDEX IF NOT EXISTS idx_omsl_created ON org_marketplace_sync_logs(created_at DESC);

-- ============================================================
-- 16. github_repo_url on org_marketplaces (safety net)
-- ============================================================
ALTER TABLE org_marketplaces ADD COLUMN IF NOT EXISTS github_repo_url TEXT;

-- ============================================================
-- 17. Activity view
--     (upstream: marketplace/048_activity_view.sql)
-- ============================================================
CREATE OR REPLACE VIEW v_all_activity AS
    (SELECT a.id, a.user_id,
        COALESCE(u.display_name, u.full_name, u.name, u.email, a.user_id) AS display_name,
        a.category, a.action, a.entity_type, a.entity_name, a.description, a.created_at,
        NOT ('anonymous' = ANY(u.roles)) AND u.email NOT LIKE '%@anonymous.local' AS is_real_user
    FROM user_activity a
    JOIN users u ON u.id = a.user_id)
    UNION ALL
    (SELECT s.id, s.user_id,
        COALESCE(u2.display_name, u2.full_name, u2.name, u2.email, s.user_id) AS display_name,
        'session' AS category,
        CASE WHEN s.ended_at IS NOT NULL THEN 'completed' ELSE 'started' END AS action,
        'session' AS entity_type, s.session_id AS entity_name,
        CASE
            WHEN s.ended_at IS NOT NULL AND sa.title IS NOT NULL AND sa.title != '' THEN sa.title
            WHEN s.ended_at IS NOT NULL AND s.ai_title IS NOT NULL AND s.ai_title != '' THEN s.ai_title
            WHEN s.ended_at IS NOT NULL THEN
                CONCAT('Completed AI session (', s.prompts, ' prompts, ', s.tool_uses, ' tool calls)')
            ELSE 'Started AI session'
        END AS description,
        COALESCE(s.ended_at, s.started_at, s.created_at) AS created_at,
        NOT ('anonymous' = ANY(u2.roles)) AND u2.email NOT LIKE '%@anonymous.local' AS is_real_user
    FROM plugin_session_summaries s
    JOIN users u2 ON u2.id = s.user_id
    LEFT JOIN session_analyses sa ON sa.session_id = s.session_id)
    UNION ALL
    (SELECT r.id, r.user_id,
        COALESCE(u3.display_name, u3.full_name, u3.name, u3.email, r.user_id) AS display_name,
        'session_rated' AS category, 'rated' AS action,
        'session' AS entity_type, r.session_id AS entity_name,
        CONCAT('Rated session ', REPEAT('★', r.rating::int), REPEAT('☆', 5 - r.rating::int)) AS description,
        r.created_at,
        NOT ('anonymous' = ANY(u3.roles)) AND u3.email NOT LIKE '%@anonymous.local' AS is_real_user
    FROM session_ratings r
    JOIN users u3 ON u3.id = r.user_id);
