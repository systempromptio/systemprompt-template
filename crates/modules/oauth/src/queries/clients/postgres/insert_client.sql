INSERT INTO oauth_clients (
    client_id, client_secret_hash, client_name,
    token_endpoint_auth_method, client_uri, logo_uri, is_active
) VALUES ($1, $2, $3, $4, $5, $6, 1)
