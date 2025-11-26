INSERT INTO task_messages (
    task_id, message_id, client_message_id, role, context_id,
    user_id, session_id, trace_id, sequence_number, metadata, reference_task_ids
) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::jsonb, $11)
