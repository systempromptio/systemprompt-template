INSERT INTO message_parts (
    message_id, task_id, part_kind, sequence_number, text_content
) VALUES ($1, $2, 'text', $3, $4)
