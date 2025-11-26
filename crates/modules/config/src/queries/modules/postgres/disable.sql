UPDATE modules
SET
    enabled = FALSE,
    updated_at = CURRENT_TIMESTAMP
WHERE name = $1
