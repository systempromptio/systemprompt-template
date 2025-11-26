INSERT INTO message_parts (
    message_id, task_id, part_kind, sequence_number,
    file_name, file_mime_type, file_uri, file_bytes
) VALUES ($1, $2, 'file', $3, $4, $5, $6, $7)
