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
