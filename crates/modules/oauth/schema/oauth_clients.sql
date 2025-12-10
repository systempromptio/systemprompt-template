-- OAuth Clients (RFC 6749 Section 2 - Client Registration)
CREATE TABLE IF NOT EXISTS oauth_clients (
    -- Core OAuth 2.0 Client Fields
    client_id TEXT PRIMARY KEY,
    client_secret_hash TEXT,
    client_name VARCHAR(255) NOT NULL,
    name VARCHAR(255) DEFAULT NULL,
    -- Client Authentication (RFC 6749 Section 2.3)
    token_endpoint_auth_method TEXT DEFAULT 'client_secret_post',
    -- Client Metadata (RFC 7591)
    client_uri TEXT,
    logo_uri TEXT,
    -- Administrative Fields
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    last_used_at TIMESTAMP
);
-- Performance index
CREATE INDEX IF NOT EXISTS idx_oauth_clients_active ON oauth_clients(is_active);
-- Update timestamps handled at application level
