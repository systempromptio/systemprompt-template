UPDATE oauth_clients
SET client_secret_hash = $1
WHERE client_id = $2 AND is_active = true
