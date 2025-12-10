-- OAuth Client Contacts Junction Table
CREATE TABLE IF NOT EXISTS oauth_client_contacts (
    client_id VARCHAR(255) NOT NULL,
    contact_email VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (client_id, contact_email),
    FOREIGN KEY (client_id) REFERENCES oauth_clients(client_id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_oauth_client_contacts_client_id
    ON oauth_client_contacts(client_id);
