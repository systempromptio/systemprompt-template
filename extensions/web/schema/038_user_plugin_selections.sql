CREATE TABLE IF NOT EXISTS user_selected_org_plugins (
    user_id TEXT NOT NULL,
    org_plugin_id TEXT NOT NULL,
    selected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, org_plugin_id)
);

CREATE INDEX IF NOT EXISTS idx_user_selected_org_plugins_user ON user_selected_org_plugins(user_id);
