SELECT client_id, client_name, created_at, updated_at, last_used_at
FROM oauth_clients
WHERE last_used_at < $1
ORDER BY last_used_at ASC
