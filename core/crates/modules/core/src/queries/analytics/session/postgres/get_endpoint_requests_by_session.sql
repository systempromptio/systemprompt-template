SELECT * FROM endpoint_requests
WHERE session_id = $1
ORDER BY requested_at ASC
