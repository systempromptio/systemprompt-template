UPDATE services SET status = $1, updated_at = CURRENT_TIMESTAMP WHERE name = $2
