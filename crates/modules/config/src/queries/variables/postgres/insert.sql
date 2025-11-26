INSERT INTO variables (
    id, name, value, type, description, category,
    is_secret, is_required, default_value, created_at
) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, CURRENT_TIMESTAMP)
