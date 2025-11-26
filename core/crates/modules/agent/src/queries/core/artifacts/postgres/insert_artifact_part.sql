INSERT INTO artifact_parts (
    artifact_id, context_id, part_kind, sequence_number,
    text_content, file_name, file_mime_type, file_uri, file_bytes,
    data_content, metadata
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::jsonb, $11::jsonb)
ON CONFLICT (artifact_id, sequence_number) DO NOTHING
