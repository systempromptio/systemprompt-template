INSERT INTO markdown_content (
    id, slug, title, description, body,
    author, published_at, keywords, kind, image, category_id, source_id, version_hash, public, parent_content_id, links
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16::jsonb);
