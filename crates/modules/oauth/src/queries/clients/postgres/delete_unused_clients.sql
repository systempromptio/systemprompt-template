DELETE FROM oauth_clients
WHERE last_used_at IS NULL
AND created_at < $1
