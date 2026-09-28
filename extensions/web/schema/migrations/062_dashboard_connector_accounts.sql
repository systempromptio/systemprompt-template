-- Public account state is separate from encrypted credentials. Disconnected
-- rows survive so every device observes a monotonically increasing revision.
CREATE SEQUENCE IF NOT EXISTS mcp_connector_revision;
CREATE TABLE IF NOT EXISTS mcp_connector_accounts (
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider TEXT NOT NULL CHECK (provider IN ('atlassian', 'github', 'salesforce')),
    status TEXT NOT NULL DEFAULT 'not_connected',
    auth_method TEXT,
    account_id TEXT,
    account_name TEXT,
    resource_id TEXT,
    resource_name TEXT,
    error_code TEXT,
    verified_at TIMESTAMPTZ,
    generation BIGINT NOT NULL DEFAULT 0,
    revision BIGINT NOT NULL DEFAULT nextval('mcp_connector_revision'),
    PRIMARY KEY (user_id, provider)
);
