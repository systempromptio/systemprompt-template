UPDATE ai_requests
SET tokens_used = $1, status = $2, error_message = $3, completed_at = CURRENT_TIMESTAMP
WHERE request_id = $4
