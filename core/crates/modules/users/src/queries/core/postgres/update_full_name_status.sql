UPDATE users SET full_name = $1, status = $2, updated_at = CURRENT_TIMESTAMP WHERE id = $3
