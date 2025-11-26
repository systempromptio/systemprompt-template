SELECT
    name, version, display_name, description,
    weight, schemas, seeds, permissions,
    enabled, created_at, updated_at
FROM modules
WHERE name = $1
