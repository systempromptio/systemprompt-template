UPDATE users
SET roles = $1,
    updated_at = CURRENT_TIMESTAMP
WHERE id = $2
