DELETE FROM oauth_clients
WHERE last_used_at < $1
