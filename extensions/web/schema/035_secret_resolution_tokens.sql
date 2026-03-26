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
