INSERT INTO markdown_content_tags (content_id, tag_id)
VALUES ($1, $2)
ON CONFLICT DO NOTHING;
