-- Migration 049: Create user_profile_reports table and fix user_hooks hook_id constraint
-- All statements are idempotent.

-- ============================================================
-- 1. user_profile_reports (AI-generated user profile reports)
-- ============================================================
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

CREATE INDEX IF NOT EXISTS idx_user_profile_reports_generated
    ON user_profile_reports(generated_at DESC);

-- ============================================================
-- 2. Fix user_hooks.hook_id NOT NULL constraint
--    The Rust code never sets hook_id — it uses the 'id' column
--    as the primary key. Make hook_id nullable with a default.
-- ============================================================
ALTER TABLE user_hooks ALTER COLUMN hook_id DROP NOT NULL;
ALTER TABLE user_hooks ALTER COLUMN hook_id SET DEFAULT '';
