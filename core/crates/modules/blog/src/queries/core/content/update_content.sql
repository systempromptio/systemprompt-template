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
    public = $12,
    links = $13::jsonb,
    updated_at = CURRENT_TIMESTAMP
WHERE id = $14;
