SELECT
    session_id,
    user_id,
    started_at,
    last_activity_at,
    ended_at,
    request_count,
    task_count,
    message_count,
    error_count,
    ai_request_count,
    total_tokens_used,
    total_ai_cost_cents,
    avg_response_time_ms,
    success_rate,
    device_type,
    browser,
    os,
    country,
    endpoints_accessed
FROM user_sessions
WHERE ended_at IS NULL
ORDER BY last_activity_at DESC