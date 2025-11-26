UPDATE users SET email = $1, full_name = $2, status = $3, updated_at = CURRENT_TIMESTAMP WHERE id = $4
