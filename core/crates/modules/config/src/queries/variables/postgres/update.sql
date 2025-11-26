UPDATE variables SET
    value = $1,
    description = $2,
    category = $3,
    is_secret = $4,
    is_required = $5,
    default_value = $6,
    updated_at = CURRENT_TIMESTAMP
WHERE id = $7
