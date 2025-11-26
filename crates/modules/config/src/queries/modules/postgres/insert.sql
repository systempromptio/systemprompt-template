INSERT INTO modules (
    name, version, display_name, description,
    weight, schemas, seeds, permissions,
    enabled, created_at
) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
ON CONFLICT(name) DO UPDATE SET
    version = excluded.version,
    display_name = excluded.display_name,
    description = excluded.description,
    weight = excluded.weight,
    schemas = excluded.schemas,
    seeds = excluded.seeds,
    permissions = excluded.permissions,
    updated_at = CURRENT_TIMESTAMP
