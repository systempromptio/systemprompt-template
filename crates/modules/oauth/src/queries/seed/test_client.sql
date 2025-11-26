-- Test OAuth client for integration testing
-- This client supports client_credentials grant for server-to-server authentication testing

BEGIN TRANSACTION;

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
    'test-debug-client',
    '$2b$12$K8x.2yqJVWx6fPFjO4uUPOyRqvO7xH7ZnJ9z5vN7zQ8tK2hF3L4Hy',
    'Test Debug Client',
    'Test Debug Client',
    'client_secret_post',
    'http://localhost:3000',
    TRUE,
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
) ON CONFLICT(client_id) DO NOTHING;

INSERT INTO oauth_client_redirect_uris (client_id, redirect_uri, is_primary)
VALUES ('test-debug-client', 'http://localhost:3000/test/callback', TRUE)
ON CONFLICT(client_id, redirect_uri) DO NOTHING;

INSERT INTO oauth_client_grant_types (client_id, grant_type)
VALUES
    ('test-debug-client', 'client_credentials'),
    ('test-debug-client', 'refresh_token')
ON CONFLICT(client_id, grant_type) DO NOTHING;

INSERT INTO oauth_client_response_types (client_id, response_type)
VALUES ('test-debug-client', 'token')
ON CONFLICT(client_id, response_type) DO NOTHING;

INSERT INTO oauth_client_scopes (client_id, scope)
VALUES
    ('test-debug-client', 'read'),
    ('test-debug-client', 'write'),
    ('test-debug-client', 'admin'),
    ('test-debug-client', 'user')
ON CONFLICT(client_id, scope) DO NOTHING;

COMMIT;
