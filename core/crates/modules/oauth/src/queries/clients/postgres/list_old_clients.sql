SELECT client_id, client_name, created_at, updated_at
FROM oauth_clients
WHERE created_at < $1
ORDER BY created_at ASC
