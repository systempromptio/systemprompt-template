SELECT id, name, slug, created_at, updated_at
FROM markdown_tags
WHERE name = $1;
