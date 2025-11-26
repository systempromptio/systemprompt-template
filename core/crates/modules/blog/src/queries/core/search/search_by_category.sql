SELECT
    c.id,
    c.title,
    c.slug,
    c.description,
    c.source_id,
    cat.name as category
FROM markdown_content c
LEFT JOIN markdown_categories cat ON c.category_id = cat.id
WHERE c.category_id = $1
ORDER BY c.published_at DESC NULLS LAST
LIMIT $2 OFFSET $3;
