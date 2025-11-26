SELECT t.id, t.name, t.slug, t.created_at
FROM markdown_tags t
INNER JOIN markdown_content_tags ct ON t.id = ct.tag_id
WHERE ct.content_id = $1;
