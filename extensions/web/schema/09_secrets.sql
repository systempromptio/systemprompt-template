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

CREATE TABLE IF NOT EXISTS skill_secrets (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    skill_id TEXT NOT NULL,
    var_name TEXT NOT NULL,
    var_value TEXT NOT NULL DEFAULT '',
    is_secret BOOLEAN NOT NULL DEFAULT true,
    encrypted_value BYTEA,
    value_nonce BYTEA,
    key_version INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, skill_id, var_name)
);
CREATE INDEX IF NOT EXISTS idx_skill_secrets_user_skill ON skill_secrets(user_id, skill_id);
CREATE INDEX IF NOT EXISTS idx_skill_secrets_user ON skill_secrets(user_id);

CREATE SCHEMA IF NOT EXISTS marketplace;

CREATE TABLE IF NOT EXISTS marketplace.magic_link_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    used BOOLEAN NOT NULL DEFAULT false,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    used_at TIMESTAMPTZ,
    ip_address TEXT
);
CREATE INDEX IF NOT EXISTS idx_magic_link_token_hash ON marketplace.magic_link_tokens(token_hash);
CREATE INDEX IF NOT EXISTS idx_magic_link_email_created ON marketplace.magic_link_tokens(email, created_at);
