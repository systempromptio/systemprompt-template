INSERT INTO oauth_clients (client_id, client_secret_hash, client_name, token_endpoint_auth_method, is_active, owner_user_id)
SELECT 'marketplace-admin', NULL, 'Marketplace Admin Dashboard', 'none', true, u.id
FROM users u
WHERE 'admin' = ANY(u.roles)
ORDER BY u.created_at ASC
LIMIT 1
ON CONFLICT (client_id) DO UPDATE SET is_active = true;

INSERT INTO oauth_client_grant_types (client_id, grant_type)
SELECT 'marketplace-admin', v.grant_type
FROM (VALUES ('authorization_code'), ('refresh_token')) AS v(grant_type)
WHERE EXISTS (SELECT 1 FROM oauth_clients WHERE client_id = 'marketplace-admin')
ON CONFLICT (client_id, grant_type) DO NOTHING;

INSERT INTO oauth_client_response_types (client_id, response_type)
SELECT 'marketplace-admin', 'code'
WHERE EXISTS (SELECT 1 FROM oauth_clients WHERE client_id = 'marketplace-admin')
ON CONFLICT (client_id, response_type) DO NOTHING;

INSERT INTO oauth_client_scopes (client_id, scope)
SELECT 'marketplace-admin', v.scope
FROM (VALUES ('admin'), ('user')) AS v(scope)
WHERE EXISTS (SELECT 1 FROM oauth_clients WHERE client_id = 'marketplace-admin')
ON CONFLICT (client_id, scope) DO NOTHING;

INSERT INTO oauth_client_redirect_uris (client_id, redirect_uri, is_primary)
SELECT 'marketplace-admin', v.redirect_uri, v.is_primary
FROM (VALUES
    ('/admin/login', true),
    ('http://localhost:8080/admin/login', false)
) AS v(redirect_uri, is_primary)
WHERE EXISTS (SELECT 1 FROM oauth_clients WHERE client_id = 'marketplace-admin')
ON CONFLICT (client_id, redirect_uri) DO NOTHING;
