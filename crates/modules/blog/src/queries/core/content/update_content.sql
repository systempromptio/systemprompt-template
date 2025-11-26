UPDATE markdown_content
SET
    title = $1,
    description = $2,
    body = $3,
    author = $4,
    published_at = $5,
    keywords = $6,
    kind = $7,
    image = $8,
    category_id = $9,
    source_id = $10,
    version_hash = $11,
    links = $12,
    updated_at = CURRENT_TIMESTAMP
WHERE id = $13;
