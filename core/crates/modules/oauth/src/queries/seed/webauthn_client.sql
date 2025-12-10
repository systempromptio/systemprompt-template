-- OAuth WebAuthn Client Seed Data
-- Web client for the SystemPrompt application

INSERT INTO oauth_clients (
    client_id,
    client_secret_hash,
    client_name,
    token_endpoint_auth_method,
    is_active,
    created_at,
    updated_at
) VALUES (
    'sp_web',
    '$2b$12$sp_web.client.secret.hash.value.for.authentication',
    'SystemPrompt Web Application',
    'none',
    true,
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
) ON CONFLICT (client_id) DO NOTHING;
