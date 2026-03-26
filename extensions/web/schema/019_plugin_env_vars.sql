CREATE TABLE IF NOT EXISTS plugin_env_vars (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    plugin_id TEXT NOT NULL,
    var_name TEXT NOT NULL,
    var_value TEXT NOT NULL DEFAULT '',
    is_secret BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, plugin_id, var_name)
);
CREATE INDEX IF NOT EXISTS idx_plugin_env_user_plugin ON plugin_env_vars(user_id, plugin_id);
