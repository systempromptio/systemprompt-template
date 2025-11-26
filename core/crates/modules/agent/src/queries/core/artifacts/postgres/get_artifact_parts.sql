SELECT
    part_kind,
    sequence_number,
    text_content,
    file_name,
    file_mime_type,
    file_uri,
    file_bytes,
    data_content::text as data_content,
    metadata
FROM artifact_parts
WHERE artifact_id = $1 AND context_id = $2
ORDER BY sequence_number ASC
