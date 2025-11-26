UPDATE user_sessions
SET
    last_activity_at = CURRENT_TIMESTAMP,
    duration_seconds = EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - started_at))::INTEGER,
    avg_response_time_ms = CASE
        WHEN request_count = 0 THEN $1
        ELSE (COALESCE(avg_response_time_ms, 0) * request_count + $2) / (request_count + 1)
    END,
    success_rate = CASE
        WHEN request_count = 0 THEN 1.0
        ELSE ((COALESCE(success_rate, 1.0) * request_count) + (1 - $3)) / (request_count + 1)
    END,
    error_count = error_count + $4,
    request_count = request_count + 1
WHERE session_id = $5