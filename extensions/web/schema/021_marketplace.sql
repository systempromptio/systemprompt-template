-- Migration 021: Marketplace ratings and visibility rules

CREATE TABLE IF NOT EXISTS plugin_ratings (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    plugin_id TEXT NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    rating SMALLINT NOT NULL CHECK (rating >= 1 AND rating <= 5),
    review TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(plugin_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_plugin_ratings_plugin ON plugin_ratings(plugin_id);
CREATE INDEX IF NOT EXISTS idx_plugin_ratings_user ON plugin_ratings(user_id);

CREATE TABLE IF NOT EXISTS plugin_visibility_rules (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    plugin_id TEXT NOT NULL,
    rule_type TEXT NOT NULL CHECK (rule_type IN ('department', 'user')),
    rule_value TEXT NOT NULL,
    access TEXT NOT NULL DEFAULT 'allow' CHECK (access IN ('allow', 'deny')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(plugin_id, rule_type, rule_value)
);

CREATE INDEX IF NOT EXISTS idx_visibility_plugin ON plugin_visibility_rules(plugin_id);
CREATE INDEX IF NOT EXISTS idx_visibility_type_value ON plugin_visibility_rules(rule_type, rule_value);
