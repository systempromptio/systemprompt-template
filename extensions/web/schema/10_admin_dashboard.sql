-- Consolidated schema: Admin dashboard, hooks catalog, billing, ratings, settings

-- hook_catalog from 032
CREATE TABLE IF NOT EXISTS hook_catalog (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    version TEXT NOT NULL DEFAULT '1.0.0',
    event TEXT NOT NULL,
    matcher TEXT NOT NULL DEFAULT '*',
    command TEXT NOT NULL DEFAULT '',
    is_async BOOLEAN NOT NULL DEFAULT false,
    category TEXT NOT NULL DEFAULT 'custom',
    enabled BOOLEAN NOT NULL DEFAULT true,
    tags TEXT[] DEFAULT '{}',
    visible_to TEXT[] DEFAULT '{}',
    checksum TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_hook_catalog_event ON hook_catalog(event);
CREATE INDEX IF NOT EXISTS idx_hook_catalog_category ON hook_catalog(category);

CREATE TABLE IF NOT EXISTS hook_plugins (
    hook_id TEXT NOT NULL REFERENCES hook_catalog(id) ON DELETE CASCADE,
    plugin_id TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (hook_id, plugin_id)
);
CREATE INDEX IF NOT EXISTS idx_hook_plugins_plugin ON hook_plugins(plugin_id);

CREATE TABLE IF NOT EXISTS hook_files (
    id TEXT PRIMARY KEY,
    hook_id TEXT NOT NULL REFERENCES hook_catalog(id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    category TEXT NOT NULL DEFAULT 'script',
    language TEXT NOT NULL DEFAULT '',
    executable BOOLEAN NOT NULL DEFAULT false,
    size_bytes BIGINT NOT NULL DEFAULT 0,
    checksum TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(hook_id, file_path)
);
CREATE INDEX IF NOT EXISTS idx_hook_files_hook ON hook_files(hook_id);

-- tenant_activity - use 046 version (has more columns)
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

-- admin_traffic_reports
CREATE TABLE IF NOT EXISTS admin_traffic_reports (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    report_date DATE NOT NULL,
    report_period TEXT NOT NULL DEFAULT 'am',
    report_data JSONB NOT NULL DEFAULT '{}'::jsonb,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (report_date, report_period)
);
CREATE INDEX IF NOT EXISTS idx_admin_traffic_reports_date ON admin_traffic_reports(report_date DESC);
CREATE INDEX IF NOT EXISTS idx_atr_date ON admin_traffic_reports(report_date DESC, generated_at DESC);

-- daily_summaries - use 047 version (composite PK)
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

-- session_analyses - use 047 version (session_id as PK)
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

-- session_entity_links
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

-- session_ratings
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

-- skill_ratings
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

-- user_achievements
CREATE TABLE IF NOT EXISTS user_achievements (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    user_id TEXT NOT NULL,
    achievement_id TEXT NOT NULL,
    unlocked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, achievement_id)
);
CREATE INDEX IF NOT EXISTS idx_ua_user ON user_achievements(user_id);
CREATE INDEX IF NOT EXISTS idx_ua_unlocked ON user_achievements(unlocked_at DESC);

-- user_ranks
CREATE TABLE IF NOT EXISTS user_ranks (
    user_id TEXT PRIMARY KEY,
    rank_name TEXT NOT NULL DEFAULT 'Spark',
    total_xp BIGINT NOT NULL DEFAULT 0,
    rank_level INT NOT NULL DEFAULT 1,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- user_settings
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

-- Marketplace schema (Paddle billing)
CREATE SCHEMA IF NOT EXISTS marketplace;

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
    role_name TEXT,
    sort_order INT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_plans_role_name ON marketplace.plans(role_name) WHERE role_name IS NOT NULL;

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

-- org_marketplace_sync_logs
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

-- user_profile_reports from 049
CREATE TABLE IF NOT EXISTS user_profile_reports (
    user_id TEXT PRIMARY KEY,
    archetype TEXT NOT NULL DEFAULT '',
    archetype_description TEXT NOT NULL DEFAULT '',
    archetype_confidence SMALLINT NOT NULL DEFAULT 0,
    strengths JSONB,
    weaknesses JSONB,
    ai_narrative TEXT,
    ai_style_analysis TEXT,
    ai_comparison TEXT,
    ai_patterns TEXT,
    ai_improvements TEXT,
    ai_tips TEXT,
    metrics_snapshot JSONB,
    period_days INT NOT NULL DEFAULT 30,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_user_profile_reports_generated ON user_profile_reports(generated_at DESC);

-- Seed default plans
INSERT INTO marketplace.plans (name, display_name, description, paddle_product_id, paddle_price_id, amount_cents, role_name, sort_order, limits)
VALUES (
    'free', 'Free', 'Default free tier', 'none', 'free_default', 0, NULL, 0,
    '{"ingestion":{"events_per_day":1000000,"content_bytes_per_day":10737418240,"sessions_per_day":10000},"entities":{"max_skills":1000,"max_agents":1000,"max_plugins":1000,"max_mcp_servers":1000,"max_hooks":1000},"features":{"ai":{"session_analysis":true,"daily_summaries":true},"apm_metrics":true,"gamification":true,"export_zip":true},"api":{"requests_per_minute":6000}}'::jsonb
)
ON CONFLICT (name) DO UPDATE SET
    limits = EXCLUDED.limits,
    display_name = EXCLUDED.display_name;

INSERT INTO marketplace.plans (name, display_name, description, paddle_product_id, paddle_price_id, amount_cents, role_name, sort_order, limits)
VALUES (
    'admin', 'Admin', 'Administrator unlimited access', 'none', 'admin_role', 0, 'admin', 99,
    '{"ingestion":{"events_per_day":9223372036854775807,"content_bytes_per_day":9223372036854775807,"sessions_per_day":9223372036854775807},"entities":{"max_skills":9223372036854775807,"max_agents":9223372036854775807,"max_plugins":9223372036854775807,"max_mcp_servers":9223372036854775807,"max_hooks":9223372036854775807},"features":{"ai":{"session_analysis":true,"daily_summaries":true},"apm_metrics":true,"gamification":true,"export_zip":true},"api":{"requests_per_minute":9223372036854775807}}'::jsonb
)
ON CONFLICT (name) DO UPDATE SET
    limits = EXCLUDED.limits,
    role_name = EXCLUDED.role_name,
    display_name = EXCLUDED.display_name;

-- v_all_activity view from 046
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
