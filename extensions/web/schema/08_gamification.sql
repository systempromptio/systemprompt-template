-- Consolidated schema: Gamification (XP, ranks, achievements, daily usage)

CREATE TABLE IF NOT EXISTS employee_xp_ledger (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    xp_amount INTEGER NOT NULL,
    source TEXT NOT NULL,
    source_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_xp_ledger_user ON employee_xp_ledger(user_id);

CREATE TABLE IF NOT EXISTS employee_ranks (
    user_id TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
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

CREATE TABLE IF NOT EXISTS employee_achievements (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    achievement_id TEXT NOT NULL,
    unlocked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, achievement_id)
);
CREATE INDEX IF NOT EXISTS idx_achievements_user ON employee_achievements(user_id);
CREATE INDEX IF NOT EXISTS idx_achievements_id ON employee_achievements(achievement_id);

CREATE TABLE IF NOT EXISTS employee_daily_usage (
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    usage_date DATE NOT NULL,
    event_count INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, usage_date)
);
CREATE INDEX IF NOT EXISTS idx_daily_usage_date ON employee_daily_usage(usage_date DESC);
