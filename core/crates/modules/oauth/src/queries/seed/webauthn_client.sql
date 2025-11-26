-- First-party web OAuth client for authorization code grant authentication
-- This client supports the OAuth 2.0 Authorization Code Grant (RFC 6749 Section 4.1) with WebAuthn
-- Client ID follows first-party pattern: sp_*

INSERT INTO oauth_clients (
    client_id,
    client_secret_hash,
    client_name,
    name,
    token_endpoint_auth_method,
    client_uri,
    is_active,
    created_at,
    updated_at
) VALUES (
    'sp_web',
    '',
    'SystemPrompt Web Application',
    'SystemPrompt Web Application',
    'none',
    'http://localhost:3000',
    TRUE,
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
) ON CONFLICT(client_id) DO NOTHING;

INSERT INTO oauth_client_redirect_uris (client_id, redirect_uri, is_primary)
VALUES
    ('sp_web', 'http://localhost:3000/auth/callback', TRUE),
    ('sp_web', 'http://localhost:8080/auth/callback', FALSE),
    ('sp_web', 'http://127.0.0.1:8080/auth/callback', FALSE)
ON CONFLICT(client_id, redirect_uri) DO NOTHING;

INSERT INTO oauth_client_grant_types (client_id, grant_type)
VALUES
    ('sp_web', 'authorization_code'),
    ('sp_web', 'refresh_token')
ON CONFLICT(client_id, grant_type) DO NOTHING;

INSERT INTO oauth_client_response_types (client_id, response_type)
VALUES ('sp_web', 'code')
ON CONFLICT(client_id, response_type) DO NOTHING;

INSERT INTO oauth_client_scopes (client_id, scope)
VALUES
    ('sp_web', 'openid'),
    ('sp_web', 'profile'),
    ('sp_web', 'email'),
    ('sp_web', 'user'),
    ('sp_web', 'admin')
ON CONFLICT(client_id, scope) DO NOTHING;
