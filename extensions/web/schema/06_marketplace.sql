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

CREATE TABLE IF NOT EXISTS marketplace_versions (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL,
    version_type TEXT NOT NULL DEFAULT 'upload' CHECK (version_type IN ('upload', 'restore', 'auto_sync')),
    snapshot_path TEXT NOT NULL,
    skills_snapshot JSONB NOT NULL,
    entities_snapshot JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, version_number)
);
CREATE INDEX IF NOT EXISTS idx_marketplace_versions_user ON marketplace_versions(user_id, version_number DESC);

CREATE TABLE IF NOT EXISTS marketplace_changelog (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    version_id TEXT REFERENCES marketplace_versions(id) ON DELETE SET NULL,
    action TEXT NOT NULL CHECK (action IN ('added', 'updated', 'deleted', 'restored')),
    skill_id TEXT NOT NULL,
    skill_name TEXT NOT NULL DEFAULT '',
    entity_type TEXT NOT NULL DEFAULT 'skill' CHECK (entity_type IN ('skill', 'agent', 'mcp_server', 'hook', 'plugin')),
    entity_id TEXT,
    entity_name TEXT NOT NULL DEFAULT '',
    detail TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_marketplace_changelog_user ON marketplace_changelog(user_id, created_at DESC);

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

CREATE TABLE IF NOT EXISTS org_marketplaces (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    enabled BOOLEAN NOT NULL DEFAULT true,
    github_repo_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
ALTER TABLE org_marketplaces ADD COLUMN IF NOT EXISTS github_repo_url TEXT;

CREATE TABLE IF NOT EXISTS org_marketplace_plugins (
    marketplace_id TEXT NOT NULL REFERENCES org_marketplaces(id) ON DELETE CASCADE,
    plugin_id TEXT NOT NULL,
    position INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (marketplace_id, plugin_id)
);
CREATE INDEX IF NOT EXISTS idx_omp_marketplace ON org_marketplace_plugins(marketplace_id);
CREATE INDEX IF NOT EXISTS idx_omp_plugin ON org_marketplace_plugins(plugin_id);

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

