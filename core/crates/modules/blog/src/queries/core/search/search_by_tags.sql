SELECT
    c.id,
    c.title,
    c.slug,
    c.description,
    c.source_id,
    cat.name as category
FROM markdown_content c
LEFT JOIN markdown_categories cat ON c.category_id = cat.id
WHERE c.id IN (
    SELECT content_id
    FROM markdown_content_tags
    WHERE tag_id IN (SELECT value FROM json_each($1))
    GROUP BY content_id
    HAVING COUNT(DISTINCT tag_id) = $2
)
ORDER BY c.published_at DESC NULLS LAST
LIMIT $3;
