UPDATE oauth_clients
SET client_name = $1, token_endpoint_auth_method = $2,
    client_uri = $3, logo_uri = $4
WHERE client_id = $5 AND is_active = true
