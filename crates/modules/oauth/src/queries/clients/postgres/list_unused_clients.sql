SELECT client_id, client_name, created_at, updated_at, last_used_at
FROM oauth_clients
WHERE last_used_at IS NULL
AND created_at < $1
ORDER BY created_at ASC
