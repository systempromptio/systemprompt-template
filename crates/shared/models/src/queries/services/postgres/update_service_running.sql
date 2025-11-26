UPDATE services SET status = 'running', pid = $1, updated_at = CURRENT_TIMESTAMP WHERE name = $2
