UPDATE markdown_content
SET image = $1,
    updated_at = $2
WHERE id = $3
