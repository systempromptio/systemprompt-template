-- Consolidated schema: Secrets, encryption, and magic links

CREATE TABLE IF NOT EXISTS plugin_env_vars (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    plugin_id TEXT NOT NULL,
    var_name TEXT NOT NULL,
    var_value TEXT NOT NULL DEFAULT '',
    is_secret BOOLEAN NOT NULL DEFAULT false,
    encrypted_value BYTEA,
    value_nonce BYTEA,
    key_version INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, plugin_id, var_name)
);
CREATE INDEX IF NOT EXISTS idx_plugin_env_user_plugin ON plugin_env_vars(user_id, plugin_id);

CREATE TABLE IF NOT EXISTS user_encryption_keys (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL UNIQUE,
    encrypted_dek BYTEA NOT NULL,
    dek_nonce BYTEA NOT NULL,
    key_version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    rotated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_user_encryption_keys_user_id ON user_encryption_keys(user_id);

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

CREATE TABLE IF NOT EXISTS secret_resolution_tokens (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    plugin_id TEXT NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_secret_resolution_tokens_hash ON secret_resolution_tokens(token_hash);
CREATE INDEX IF NOT EXISTS idx_secret_resolution_tokens_expires ON secret_resolution_tokens(expires_at);

-- Shared by the plans/billing tables in 10_admin_dashboard.sql.
CREATE SCHEMA IF NOT EXISTS marketplace;
