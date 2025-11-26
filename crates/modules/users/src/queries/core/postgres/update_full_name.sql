UPDATE users SET full_name = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2
