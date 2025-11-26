UPDATE modules SET
    version = $1,
    display_name = $2,
    description = $3,
    weight = $4,
    schemas = $5,
    seeds = $6,
    permissions = $7,
    updated_at = CURRENT_TIMESTAMP
WHERE name = $8
