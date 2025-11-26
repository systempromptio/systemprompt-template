UPDATE services SET status = 'error', pid = NULL, updated_at = CURRENT_TIMESTAMP WHERE name = ?
