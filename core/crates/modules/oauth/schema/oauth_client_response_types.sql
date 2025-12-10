-- OAuth Client Response Types Junction Table
CREATE TABLE IF NOT EXISTS oauth_client_response_types (
    client_id VARCHAR(255) NOT NULL,
    response_type VARCHAR(255) NOT NULL CHECK (response_type IN ('code', 'token')),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (client_id, response_type),
    FOREIGN KEY (client_id) REFERENCES oauth_clients(client_id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_oauth_client_response_types_client_id
    ON oauth_client_response_types(client_id);
