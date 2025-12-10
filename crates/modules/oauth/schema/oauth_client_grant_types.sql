-- OAuth Client Grant Types Junction Table
CREATE TABLE IF NOT EXISTS oauth_client_grant_types (
    client_id VARCHAR(255) NOT NULL,
    grant_type VARCHAR(255) NOT NULL CHECK (grant_type IN (
        'authorization_code', 'refresh_token', 'client_credentials', 'password'
    )),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (client_id, grant_type),
    FOREIGN KEY (client_id) REFERENCES oauth_clients(client_id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_oauth_client_grant_types_client_id
    ON oauth_client_grant_types(client_id);
CREATE INDEX IF NOT EXISTS idx_oauth_client_grant_types_type
    ON oauth_client_grant_types(grant_type);
