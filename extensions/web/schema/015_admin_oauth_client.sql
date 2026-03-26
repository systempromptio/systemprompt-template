INSERT INTO oauth_clients (client_id, client_secret_hash, client_name, token_endpoint_auth_method, is_active)
VALUES ('marketplace-admin', NULL, 'Marketplace Admin Dashboard', 'none', true)
ON CONFLICT (client_id) DO NOTHING;

INSERT INTO oauth_client_grant_types (client_id, grant_type)
VALUES ('marketplace-admin', 'authorization_code')
ON CONFLICT (client_id, grant_type) DO NOTHING;

INSERT INTO oauth_client_grant_types (client_id, grant_type)
VALUES ('marketplace-admin', 'refresh_token')
ON CONFLICT (client_id, grant_type) DO NOTHING;

INSERT INTO oauth_client_response_types (client_id, response_type)
VALUES ('marketplace-admin', 'code')
ON CONFLICT (client_id, response_type) DO NOTHING;

INSERT INTO oauth_client_scopes (client_id, scope)
VALUES ('marketplace-admin', 'admin')
ON CONFLICT (client_id, scope) DO NOTHING;

INSERT INTO oauth_client_redirect_uris (client_id, redirect_uri, is_primary)
VALUES ('marketplace-admin', '/admin/login', true)
ON CONFLICT (client_id, redirect_uri) DO NOTHING;

INSERT INTO oauth_client_redirect_uris (client_id, redirect_uri, is_primary)
VALUES ('marketplace-admin', 'http://localhost:8080/admin/login', false)
ON CONFLICT (client_id, redirect_uri) DO NOTHING;

INSERT INTO oauth_client_redirect_uris (client_id, redirect_uri, is_primary)
VALUES ('marketplace-admin', 'https://f7ae798f9c2a.systemprompt.io/admin/login', false)
ON CONFLICT (client_id, redirect_uri) DO NOTHING;
