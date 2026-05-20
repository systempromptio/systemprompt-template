-- Re-seed marketplace-admin after core migration 004_oauth_client_owner
-- (which deleted all oauth_clients and added NOT NULL owner_user_id
-- REFERENCES users(id)). Bootstraps a synthetic 'system' user so the client
-- has a valid owner before any human admin exists.

BEGIN;

INSERT INTO users (id, name, email, status, email_verified, roles)
VALUES ('system', 'system', 'system@local', 'active', true, ARRAY['admin', 'user']::TEXT[])
ON CONFLICT (id) DO NOTHING;

INSERT INTO oauth_clients (
    client_id, client_secret_hash, client_name,
    token_endpoint_auth_method, is_active, owner_user_id
)
VALUES (
    'marketplace-admin', NULL, 'Marketplace Admin Dashboard',
    'none', true, 'system'
)
ON CONFLICT (client_id) DO UPDATE
   SET owner_user_id = EXCLUDED.owner_user_id,
       is_active     = true;

INSERT INTO oauth_client_grant_types (client_id, grant_type) VALUES
    ('marketplace-admin', 'authorization_code'),
    ('marketplace-admin', 'refresh_token')
ON CONFLICT (client_id, grant_type) DO NOTHING;

INSERT INTO oauth_client_response_types (client_id, response_type)
VALUES ('marketplace-admin', 'code')
ON CONFLICT (client_id, response_type) DO NOTHING;

INSERT INTO oauth_client_scopes (client_id, scope) VALUES
    ('marketplace-admin', 'admin'),
    ('marketplace-admin', 'user')
ON CONFLICT (client_id, scope) DO NOTHING;

INSERT INTO oauth_client_redirect_uris (client_id, redirect_uri, is_primary) VALUES
    ('marketplace-admin', '/admin/login', true),
    ('marketplace-admin', 'http://localhost:8080/admin/login', false),
    ('marketplace-admin', 'https://f7ae798f9c2a.systemprompt.io/admin/login', false)
ON CONFLICT (client_id, redirect_uri) DO NOTHING;

COMMIT;
