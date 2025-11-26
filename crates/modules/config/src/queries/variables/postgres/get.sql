SELECT
    id, name, value, type, description, category,
    is_secret, is_required, default_value,
    created_at, updated_at
FROM variables
WHERE name = $1
