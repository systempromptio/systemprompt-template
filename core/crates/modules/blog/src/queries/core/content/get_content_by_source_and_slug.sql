SELECT
    id, slug, title, description, body,
    author, published_at, keywords, kind, image,
    category_id, source_id, version_hash, public, links,
    created_at, updated_at
FROM markdown_content
WHERE source_id = $1 AND slug = $2;
