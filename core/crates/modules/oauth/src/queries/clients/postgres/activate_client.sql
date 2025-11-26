UPDATE oauth_clients
SET is_active = TRUE
WHERE client_id = $1
