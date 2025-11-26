-- Delete all content for a specific source
DELETE FROM markdown_content
WHERE source_id = $1;
