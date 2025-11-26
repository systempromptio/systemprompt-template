-- WebAuthn credentials - Passwordless authentication storage
CREATE TABLE IF NOT EXISTS webauthn_credentials (
    id TEXT PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL,
    -- WebAuthn credential data
    credential_id BYTEA NOT NULL UNIQUE,
    public_key BYTEA NOT NULL,
    counter INTEGER NOT NULL DEFAULT 0,
    -- User-facing metadata
    display_name VARCHAR(255) NOT NULL,
    device_type TEXT CHECK(device_type IN ('platform', 'cross-platform')) DEFAULT 'platform',
    transports TEXT DEFAULT '["internal"]',
    -- Timestamps
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_used_at TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
-- Performance indexes
CREATE INDEX IF NOT EXISTS idx_webauthn_credentials_user_id ON webauthn_credentials(user_id);
CREATE INDEX IF NOT EXISTS idx_webauthn_credentials_credential_id ON webauthn_credentials(credential_id);
CREATE INDEX IF NOT EXISTS idx_webauthn_credentials_last_used ON webauthn_credentials(last_used_at);