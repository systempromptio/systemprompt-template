INSERT INTO endpoint_requests (
    session_id,
    endpoint_path,
    http_method,
    response_status,
    response_time_ms
)
SELECT $1, $2, $3, $4, $5
WHERE EXISTS (
    SELECT 1 FROM user_sessions WHERE session_id = $1
)
