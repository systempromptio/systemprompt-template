-- OAuth Test Client Seed Data
-- Test client for development and testing

INSERT INTO oauth_clients (
    client_id,
    client_secret_hash,
    client_name,
    token_endpoint_auth_method,
    is_active,
    created_at,
    updated_at
) VALUES (
    'test-debug-client',
    '$2b$12$test.debug.client.secret.hash.value.for.testing',
    'Test Debug Client',
    'client_secret_post',
    true,
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
) ON CONFLICT (client_id) DO NOTHING;
