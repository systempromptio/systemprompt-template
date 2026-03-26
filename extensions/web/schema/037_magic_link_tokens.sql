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

CREATE INDEX IF NOT EXISTS idx_magic_link_token_hash
    ON marketplace.magic_link_tokens(token_hash);
CREATE INDEX IF NOT EXISTS idx_magic_link_email_created
    ON marketplace.magic_link_tokens(email, created_at);
