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
