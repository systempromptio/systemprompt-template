UPDATE oauth_clients
SET is_active = false, updated_at = EXTRACT(EPOCH FROM NOW())::bigint
WHERE created_at < $1
AND (client_name LIKE 'Test%' OR client_name LIKE '%Test%')
AND is_active = true
