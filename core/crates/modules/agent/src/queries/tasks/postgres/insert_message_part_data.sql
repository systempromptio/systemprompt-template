INSERT INTO message_parts (
    message_id, task_id, part_kind, sequence_number, data_content
) VALUES ($1, $2, 'data', $3, $4::jsonb)
