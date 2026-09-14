-- Maps a systemprompt user to their Salesforce Username, one row per org.
-- `provider` is the MCP server id fronting the org (`salesforce`,
-- `salesforce-<slug>`).
--
-- The RFC 7523 JWT-bearer assertion signs `sub = <Salesforce Username>`, which
-- is NOT the login email (it looks like ed.aa...@agentforce.com). Salesforce
-- SSO used to capture it from the userinfo `preferred_username` claim at login;
-- ADFS replaced that login, so the mapping is now set administratively.
--
-- No token is stored: the JWT-bearer grant mints a fresh bearer on every
-- accessor call, so there is nothing here worth stealing beyond the username.

CREATE TABLE IF NOT EXISTS salesforce_user_identities (
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider    TEXT NOT NULL DEFAULT 'salesforce'
                CHECK (provider ~ '^salesforce(-[A-Za-z0-9_-]{1,117})?$'),
    sf_username TEXT NOT NULL,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, provider)
);
