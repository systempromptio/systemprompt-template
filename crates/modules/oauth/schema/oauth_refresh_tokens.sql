CREATE TABLE IF NOT EXISTS oauth_refresh_tokens (
    token_id TEXT PRIMARY KEY,
    client_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    scope TEXT NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (client_id) REFERENCES oauth_clients(client_id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_lookup ON oauth_refresh_tokens(token_id, expires_at);