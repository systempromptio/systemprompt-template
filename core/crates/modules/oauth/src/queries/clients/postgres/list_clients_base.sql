SELECT
    client_id, client_secret_hash, client_name, name,
    token_endpoint_auth_method, client_uri, logo_uri,
    is_active, created_at, updated_at
FROM oauth_clients
WHERE is_active = TRUE
ORDER BY created_at DESC
