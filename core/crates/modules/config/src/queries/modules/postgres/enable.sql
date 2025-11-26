UPDATE modules
SET
    enabled = TRUE,
    updated_at = CURRENT_TIMESTAMP
WHERE name = $1
