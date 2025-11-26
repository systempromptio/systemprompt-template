UPDATE oauth_clients
SET last_used_at = $1, updated_at = EXTRACT(EPOCH FROM NOW())::bigint
WHERE client_id = $2
