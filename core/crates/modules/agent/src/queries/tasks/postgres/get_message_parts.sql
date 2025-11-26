SELECT * FROM message_parts
WHERE message_id = $1
ORDER BY sequence_number ASC
