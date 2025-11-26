UPDATE services SET status = 'stopped', pid = NULL, updated_at = CURRENT_TIMESTAMP WHERE name = $1
