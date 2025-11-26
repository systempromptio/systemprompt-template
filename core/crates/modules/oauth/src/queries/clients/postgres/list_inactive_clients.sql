SELECT client_id, client_name, created_at, updated_at
FROM oauth_clients
WHERE is_active = false
ORDER BY updated_at DESC
