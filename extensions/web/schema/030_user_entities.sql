-- Migration 030: User-level plugins, MCP servers, and marketplace sync
-- Two-tier entity management: user-owned DB entities alongside org-level YAML configs

-- 1. user_plugins: user-owned plugin bundles
CREATE TABLE IF NOT EXISTS user_plugins (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    plugin_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    version TEXT NOT NULL DEFAULT '1.0.0',
    enabled BOOLEAN NOT NULL DEFAULT true,
    category TEXT NOT NULL DEFAULT '',
    keywords TEXT[] DEFAULT '{}',
    author_name TEXT NOT NULL DEFAULT '',
    base_plugin_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, plugin_id)
);

CREATE INDEX IF NOT EXISTS idx_user_plugins_user_id ON user_plugins(user_id);
CREATE INDEX IF NOT EXISTS idx_user_plugins_plugin_id ON user_plugins(plugin_id);

-- 2. user_mcp_servers: user-owned MCP server configs
CREATE TABLE IF NOT EXISTS user_mcp_servers (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    mcp_server_id TEXT NOT NULL,
    name TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    "binary" TEXT NOT NULL DEFAULT '',
    package_name TEXT NOT NULL DEFAULT '',
    port INTEGER NOT NULL DEFAULT 0,
    endpoint TEXT NOT NULL DEFAULT '',
    enabled BOOLEAN NOT NULL DEFAULT true,
    oauth_required BOOLEAN NOT NULL DEFAULT false,
    oauth_scopes TEXT[] DEFAULT '{}',
    oauth_audience TEXT NOT NULL DEFAULT '',
    base_mcp_server_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, mcp_server_id)
);

CREATE INDEX IF NOT EXISTS idx_user_mcp_servers_user_id ON user_mcp_servers(user_id);
CREATE INDEX IF NOT EXISTS idx_user_mcp_servers_mcp_server_id ON user_mcp_servers(mcp_server_id);

-- 3. Plugin-entity association tables
CREATE TABLE IF NOT EXISTS user_plugin_skills (
    user_plugin_id TEXT NOT NULL REFERENCES user_plugins(id) ON DELETE CASCADE,
    user_skill_id TEXT NOT NULL REFERENCES user_skills(id) ON DELETE CASCADE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_plugin_id, user_skill_id)
);

CREATE INDEX IF NOT EXISTS idx_user_plugin_skills_plugin ON user_plugin_skills(user_plugin_id);
CREATE INDEX IF NOT EXISTS idx_user_plugin_skills_skill ON user_plugin_skills(user_skill_id);

CREATE TABLE IF NOT EXISTS user_plugin_agents (
    user_plugin_id TEXT NOT NULL REFERENCES user_plugins(id) ON DELETE CASCADE,
    user_agent_id TEXT NOT NULL REFERENCES user_agents(id) ON DELETE CASCADE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_plugin_id, user_agent_id)
);

CREATE INDEX IF NOT EXISTS idx_user_plugin_agents_plugin ON user_plugin_agents(user_plugin_id);
CREATE INDEX IF NOT EXISTS idx_user_plugin_agents_agent ON user_plugin_agents(user_agent_id);

CREATE TABLE IF NOT EXISTS user_plugin_mcp_servers (
    user_plugin_id TEXT NOT NULL REFERENCES user_plugins(id) ON DELETE CASCADE,
    user_mcp_server_id TEXT NOT NULL REFERENCES user_mcp_servers(id) ON DELETE CASCADE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_plugin_id, user_mcp_server_id)
);

CREATE INDEX IF NOT EXISTS idx_user_plugin_mcp_servers_plugin ON user_plugin_mcp_servers(user_plugin_id);
CREATE INDEX IF NOT EXISTS idx_user_plugin_mcp_servers_mcp ON user_plugin_mcp_servers(user_mcp_server_id);

CREATE TABLE IF NOT EXISTS user_plugin_hooks (
    user_plugin_id TEXT NOT NULL REFERENCES user_plugins(id) ON DELETE CASCADE,
    user_hook_id TEXT NOT NULL REFERENCES user_hooks(id) ON DELETE CASCADE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_plugin_id, user_hook_id)
);

CREATE INDEX IF NOT EXISTS idx_user_plugin_hooks_plugin ON user_plugin_hooks(user_plugin_id);
CREATE INDEX IF NOT EXISTS idx_user_plugin_hooks_hook ON user_plugin_hooks(user_hook_id);

-- 4. Marketplace sync tracking
CREATE TABLE IF NOT EXISTS marketplace_sync_status (
    user_id TEXT PRIMARY KEY,
    dirty BOOLEAN NOT NULL DEFAULT true,
    last_synced_at TIMESTAMPTZ,
    last_changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sync_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_marketplace_sync_status_dirty ON marketplace_sync_status(dirty) WHERE dirty = true;

-- 5. DB triggers to auto-mark dirty on entity changes
CREATE OR REPLACE FUNCTION mark_marketplace_dirty() RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO marketplace_sync_status (user_id, dirty, last_changed_at)
    VALUES (COALESCE(NEW.user_id, OLD.user_id), true, NOW())
    ON CONFLICT (user_id) DO UPDATE SET dirty = true, last_changed_at = NOW();
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Triggers on all user entity tables
DROP TRIGGER IF EXISTS trg_user_skills_sync ON user_skills;
CREATE TRIGGER trg_user_skills_sync AFTER INSERT OR UPDATE OR DELETE ON user_skills FOR EACH ROW EXECUTE FUNCTION mark_marketplace_dirty();
DROP TRIGGER IF EXISTS trg_user_agents_sync ON user_agents;
CREATE TRIGGER trg_user_agents_sync AFTER INSERT OR UPDATE OR DELETE ON user_agents FOR EACH ROW EXECUTE FUNCTION mark_marketplace_dirty();
DROP TRIGGER IF EXISTS trg_user_hooks_sync ON user_hooks;
CREATE TRIGGER trg_user_hooks_sync AFTER INSERT OR UPDATE OR DELETE ON user_hooks FOR EACH ROW EXECUTE FUNCTION mark_marketplace_dirty();
DROP TRIGGER IF EXISTS trg_user_plugins_sync ON user_plugins;
CREATE TRIGGER trg_user_plugins_sync AFTER INSERT OR UPDATE OR DELETE ON user_plugins FOR EACH ROW EXECUTE FUNCTION mark_marketplace_dirty();
DROP TRIGGER IF EXISTS trg_user_mcp_servers_sync ON user_mcp_servers;
CREATE TRIGGER trg_user_mcp_servers_sync AFTER INSERT OR UPDATE OR DELETE ON user_mcp_servers FOR EACH ROW EXECUTE FUNCTION mark_marketplace_dirty();

-- 6. Broaden marketplace_versions for multi-entity snapshots
ALTER TABLE marketplace_versions ADD COLUMN IF NOT EXISTS entities_snapshot JSONB;
ALTER TABLE marketplace_changelog ADD COLUMN IF NOT EXISTS entity_type TEXT NOT NULL DEFAULT 'skill' CHECK (entity_type IN ('skill', 'agent', 'mcp_server', 'hook', 'plugin'));
ALTER TABLE marketplace_changelog ADD COLUMN IF NOT EXISTS entity_id TEXT;
ALTER TABLE marketplace_changelog ADD COLUMN IF NOT EXISTS entity_name TEXT NOT NULL DEFAULT '';
UPDATE marketplace_changelog SET entity_id = skill_id, entity_name = skill_name WHERE entity_id IS NULL;

-- Update version_type to allow auto_sync
ALTER TABLE marketplace_versions DROP CONSTRAINT IF EXISTS marketplace_versions_version_type_check;
ALTER TABLE marketplace_versions ADD CONSTRAINT marketplace_versions_version_type_check CHECK (version_type IN ('upload', 'restore', 'auto_sync'));
