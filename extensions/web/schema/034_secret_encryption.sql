-- Migration 034: Add encryption columns to plugin_env_vars and create audit log

-- Add encryption columns to existing plugin_env_vars table
ALTER TABLE plugin_env_vars ADD COLUMN IF NOT EXISTS encrypted_value BYTEA;
ALTER TABLE plugin_env_vars ADD COLUMN IF NOT EXISTS value_nonce BYTEA;
ALTER TABLE plugin_env_vars ADD COLUMN IF NOT EXISTS key_version INTEGER NOT NULL DEFAULT 0;

-- Audit log for secret operations
CREATE TABLE IF NOT EXISTS secret_audit_log (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    plugin_id TEXT NOT NULL,
    var_name TEXT NOT NULL,
    action TEXT NOT NULL CHECK (action IN ('created', 'updated', 'accessed', 'rotated', 'deleted')),
    actor_id TEXT NOT NULL,
    ip_address TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_secret_audit_log_user_created ON secret_audit_log(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_secret_audit_log_user_plugin ON secret_audit_log(user_id, plugin_id);
