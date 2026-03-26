INSERT INTO oauth_client_scopes (client_id, scope)
VALUES ('marketplace-admin', 'user')
ON CONFLICT (client_id, scope) DO NOTHING;
