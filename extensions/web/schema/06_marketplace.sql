-- Consolidated schema: Marketplace, ratings, access control, installations

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

-- marketplace_versions / marketplace_changelog: removed.
-- Marketplaces are now defined as YAML at services/marketplaces/<id>/config.yaml
-- and ingested by core into ServicesConfig.marketplaces at boot. There is no
-- runtime authoring surface, so version snapshots and per-user changelogs are
-- no longer needed. Drop any pre-existing tables left over from older deployments.
DROP TABLE IF EXISTS marketplace_changelog CASCADE;
DROP TABLE IF EXISTS marketplace_versions CASCADE;

CREATE TABLE IF NOT EXISTS access_control_rules (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    entity_type TEXT NOT NULL CHECK (entity_type IN ('plugin', 'agent', 'mcp_server', 'marketplace')),
    entity_id TEXT NOT NULL,
    rule_type TEXT NOT NULL CHECK (rule_type IN ('role', 'department')),
    rule_value TEXT NOT NULL,
    access TEXT NOT NULL DEFAULT 'allow' CHECK (access IN ('allow', 'deny')),
    default_included BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(entity_type, entity_id, rule_type, rule_value)
);
CREATE INDEX IF NOT EXISTS idx_acl_entity ON access_control_rules(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_acl_rule ON access_control_rules(rule_type, rule_value);
CREATE INDEX IF NOT EXISTS idx_acl_default ON access_control_rules(default_included) WHERE default_included = true;

INSERT INTO access_control_rules (id, entity_type, entity_id, rule_type, rule_value, access, created_at)
SELECT id, 'plugin', plugin_id, rule_type, rule_value, access, created_at
FROM plugin_visibility_rules
ON CONFLICT DO NOTHING;

-- org_marketplaces / org_marketplace_plugins: removed.
-- Marketplaces are YAML-defined in services/marketplaces/. The DB no longer
-- owns marketplace definitions. See core's ServicesConfig.marketplaces.
DROP TABLE IF EXISTS org_marketplace_plugins CASCADE;
DROP TABLE IF EXISTS org_marketplaces CASCADE;

CREATE TABLE IF NOT EXISTS plugin_installations (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    plugin_id TEXT NOT NULL,
    plugin_version TEXT NOT NULL,
    plugin_source TEXT NOT NULL DEFAULT 'org',
    base_plugin_id TEXT,
    first_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    session_count INT NOT NULL DEFAULT 1,
    UNIQUE(user_id, plugin_id)
);
CREATE INDEX IF NOT EXISTS idx_plugin_installations_user ON plugin_installations(user_id);
CREATE INDEX IF NOT EXISTS idx_plugin_installations_plugin ON plugin_installations(plugin_id);
CREATE INDEX IF NOT EXISTS idx_plugin_installations_source ON plugin_installations(plugin_source);
CREATE INDEX IF NOT EXISTS idx_plugin_installations_base ON plugin_installations(base_plugin_id) WHERE base_plugin_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS plugin_installation_history (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    plugin_id TEXT NOT NULL,
    plugin_version TEXT NOT NULL,
    event_type TEXT NOT NULL,
    previous_version TEXT,
    plugin_source TEXT NOT NULL DEFAULT 'org',
    base_plugin_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_install_history_user ON plugin_installation_history(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_install_history_plugin ON plugin_installation_history(plugin_id, created_at DESC);

