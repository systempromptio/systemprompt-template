-- Hook catalog: first-class hook definitions (mirrors agent_skills pattern)
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

-- Hook-to-plugin association (replaces embedded hooks in plugin config.yaml)
CREATE TABLE IF NOT EXISTS hook_plugins (
    hook_id TEXT NOT NULL REFERENCES hook_catalog(id) ON DELETE CASCADE,
    plugin_id TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (hook_id, plugin_id)
);
CREATE INDEX IF NOT EXISTS idx_hook_plugins_plugin ON hook_plugins(plugin_id);

-- Hook file inventory (mirrors skill_files pattern for sync)
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
