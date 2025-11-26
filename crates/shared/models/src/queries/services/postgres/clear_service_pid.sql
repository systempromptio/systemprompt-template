UPDATE services SET pid = NULL, updated_at = CURRENT_TIMESTAMP WHERE name = $1
