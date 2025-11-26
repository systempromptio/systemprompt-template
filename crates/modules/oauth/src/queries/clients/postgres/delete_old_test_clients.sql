DELETE FROM oauth_clients
WHERE created_at < $1
AND client_name LIKE 'Test%'
