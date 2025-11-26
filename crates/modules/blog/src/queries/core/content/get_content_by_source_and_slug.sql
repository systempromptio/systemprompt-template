SELECT
    id, slug, title, description, body,
    author, published_at, keywords, kind, image,
    category_id, source_id, version_hash, public, parent_content_id, links,
    created_at, updated_at
FROM markdown_content
WHERE source_id = $1 AND slug = $2;
