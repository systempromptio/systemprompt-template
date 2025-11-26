UPDATE oauth_clients
SET is_active = FALSE
WHERE client_id = $1
