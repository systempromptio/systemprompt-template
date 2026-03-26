CREATE TABLE IF NOT EXISTS plugin_installations (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    plugin_id TEXT NOT NULL,
    plugin_version TEXT NOT NULL,
    plugin_source TEXT NOT NULL DEFAULT 'org',  -- 'org' or 'custom'
    base_plugin_id TEXT,                         -- org plugin this was forked from (NULL for org plugins)
    first_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    session_count INT NOT NULL DEFAULT 1,
    UNIQUE(user_id, plugin_id)
);

CREATE INDEX idx_plugin_installations_user ON plugin_installations(user_id);
CREATE INDEX idx_plugin_installations_plugin ON plugin_installations(plugin_id);
CREATE INDEX idx_plugin_installations_source ON plugin_installations(plugin_source);
CREATE INDEX idx_plugin_installations_base ON plugin_installations(base_plugin_id) WHERE base_plugin_id IS NOT NULL;

-- History of version changes
CREATE TABLE IF NOT EXISTS plugin_installation_history (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    plugin_id TEXT NOT NULL,
    plugin_version TEXT NOT NULL,
    event_type TEXT NOT NULL,  -- 'installed' or 'updated'
    previous_version TEXT,
    plugin_source TEXT NOT NULL DEFAULT 'org',
    base_plugin_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_install_history_user ON plugin_installation_history(user_id, created_at DESC);
CREATE INDEX idx_install_history_plugin ON plugin_installation_history(plugin_id, created_at DESC);
