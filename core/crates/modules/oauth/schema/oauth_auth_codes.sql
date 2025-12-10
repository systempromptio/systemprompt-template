-- Authorization Codes (RFC 6749 Section 4.1 - Authorization Code Grant)
CREATE TABLE IF NOT EXISTS oauth_auth_codes (
    -- Core OAuth 2.0 Authorization Code Fields
    code VARCHAR(255) PRIMARY KEY,                                         -- RFC 6749 Section 4.1.2 - REQUIRED
    client_id VARCHAR(255) NOT NULL,                                       -- RFC 6749 Section 4.1.3 - REQUIRED
    user_id VARCHAR(255) NOT NULL,                                         -- Resource owner identifier
    redirect_uri TEXT NOT NULL,                                    -- RFC 6749 Section 4.1.3 - REQUIRED if in auth request
    scope TEXT NOT NULL,                                           -- RFC 6749 Section 4.1.3 - Granted scope
    expires_at TIMESTAMPTZ NOT NULL,                                 -- RFC 6749 Section 4.1.2 - Short-lived (≤10 minutes)
    -- PKCE Extension (RFC 7636 - Proof Key for Code Exchange)
    code_challenge TEXT,                                           -- RFC 7636 Section 4.3
    code_challenge_method TEXT,                                    -- "S256" or "plain"
    -- OpenID Connect Extension
    nonce TEXT,                                                    -- OIDC Section 3.1.2.1 - Replay protection
    -- Administrative Fields
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,                -- Creation timestamp
    used_at TIMESTAMPTZ,                                             -- Single-use enforcement
    -- Foreign Key Constraints
    FOREIGN KEY (client_id) REFERENCES oauth_clients(client_id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    -- Constraints
    CHECK (expires_at > created_at),                               -- Must expire after creation
    CHECK (code_challenge_method IN ('S256', 'plain') OR code_challenge_method IS NULL)
);
-- Performance indexes
CREATE INDEX IF NOT EXISTS idx_auth_codes_expires ON oauth_auth_codes(expires_at);
CREATE INDEX IF NOT EXISTS idx_auth_codes_user ON oauth_auth_codes(user_id);
CREATE INDEX IF NOT EXISTS idx_auth_codes_client ON oauth_auth_codes(client_id);
CREATE INDEX IF NOT EXISTS idx_auth_codes_lookup ON oauth_auth_codes(code, expires_at);
-- Note: Expired code cleanup handled at application level or via scheduled maintenance