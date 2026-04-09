-- Consolidated schema: User entities (skills, agents, hooks, plugins, MCP servers)

CREATE TABLE IF NOT EXISTS user_skills (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    skill_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    content TEXT NOT NULL DEFAULT '',
    enabled BOOLEAN NOT NULL DEFAULT true,
    version TEXT NOT NULL DEFAULT '1.0.0',
    tags TEXT[] DEFAULT '{}',
    base_skill_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, skill_id)
);
CREATE INDEX IF NOT EXISTS idx_user_skills_user ON user_skills(user_id);
CREATE INDEX IF NOT EXISTS idx_user_skills_enabled ON user_skills(user_id, enabled);
CREATE INDEX IF NOT EXISTS idx_user_skills_base ON user_skills(base_skill_id);

CREATE TABLE IF NOT EXISTS skill_files (
    id TEXT PRIMARY KEY,
    skill_id TEXT NOT NULL,
    file_path TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    category TEXT NOT NULL DEFAULT 'script',
    language TEXT NOT NULL DEFAULT '',
    executable BOOLEAN NOT NULL DEFAULT false,
    size_bytes BIGINT NOT NULL DEFAULT 0,
    checksum TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(skill_id, file_path)
);
CREATE INDEX IF NOT EXISTS idx_skill_files_skill ON skill_files(skill_id);
CREATE INDEX IF NOT EXISTS idx_skill_files_category ON skill_files(skill_id, category);

CREATE TABLE IF NOT EXISTS user_agents (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    system_prompt TEXT NOT NULL DEFAULT '',
    enabled BOOLEAN NOT NULL DEFAULT true,
    base_agent_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, agent_id)
);
CREATE INDEX IF NOT EXISTS idx_user_agents_user ON user_agents(user_id);
CREATE INDEX IF NOT EXISTS idx_user_agents_base ON user_agents(base_agent_id);

CREATE TABLE IF NOT EXISTS hook_overrides (
    hook_id TEXT PRIMARY KEY,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_hooks (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    hook_id TEXT DEFAULT '',
    name TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    event TEXT DEFAULT '',
    matcher TEXT NOT NULL DEFAULT '*',
    command TEXT NOT NULL,
    is_async BOOLEAN NOT NULL DEFAULT false,
    enabled BOOLEAN NOT NULL DEFAULT true,
    base_hook_id TEXT,
    plugin_id TEXT,
    is_default BOOLEAN NOT NULL DEFAULT false,
    hook_name TEXT NOT NULL DEFAULT '',
    event_type TEXT NOT NULL DEFAULT '',
    hook_type TEXT NOT NULL DEFAULT 'http',
    url TEXT NOT NULL DEFAULT '',
    headers JSONB NOT NULL DEFAULT '{}',
    timeout INT NOT NULL DEFAULT 10,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, hook_id)
);
CREATE INDEX IF NOT EXISTS idx_user_hooks_user ON user_hooks(user_id);
CREATE INDEX IF NOT EXISTS idx_user_hooks_base ON user_hooks(base_hook_id);
CREATE INDEX IF NOT EXISTS idx_user_hooks_plugin ON user_hooks(plugin_id);
CREATE INDEX IF NOT EXISTS idx_user_hooks_event_type ON user_hooks(event_type);
CREATE INDEX IF NOT EXISTS idx_user_hooks_default ON user_hooks(user_id, is_default) WHERE is_default = true;

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

CREATE TABLE IF NOT EXISTS marketplace_sync_status (
    user_id TEXT PRIMARY KEY,
    dirty BOOLEAN NOT NULL DEFAULT true,
    last_synced_at TIMESTAMPTZ,
    last_changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sync_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_marketplace_sync_status_dirty ON marketplace_sync_status(dirty) WHERE dirty = true;

CREATE OR REPLACE FUNCTION mark_marketplace_dirty() RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO marketplace_sync_status (user_id, dirty, last_changed_at)
    VALUES (COALESCE(NEW.user_id, OLD.user_id), true, NOW())
    ON CONFLICT (user_id) DO UPDATE SET dirty = true, last_changed_at = NOW();
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

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

CREATE TABLE IF NOT EXISTS user_selected_org_plugins (
    user_id TEXT NOT NULL,
    org_plugin_id TEXT NOT NULL,
    selected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, org_plugin_id)
);
CREATE INDEX IF NOT EXISTS idx_user_selected_org_plugins_user ON user_selected_org_plugins(user_id);
