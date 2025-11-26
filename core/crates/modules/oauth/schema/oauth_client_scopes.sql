-- OAuth Client Scopes Junction Table
CREATE TABLE IF NOT EXISTS oauth_client_scopes (
    client_id VARCHAR(255) NOT NULL,
    scope TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (client_id, scope),
    FOREIGN KEY (client_id) REFERENCES oauth_clients(client_id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_oauth_client_scopes_client_id ON oauth_client_scopes(client_id);
CREATE INDEX IF NOT EXISTS idx_oauth_client_scopes_scope ON oauth_client_scopes(scope);
