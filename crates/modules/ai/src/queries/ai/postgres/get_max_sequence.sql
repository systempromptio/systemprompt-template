SELECT COALESCE(MAX(sequence_number), -1) as max_seq
FROM ai_request_messages
WHERE request_id = $1
