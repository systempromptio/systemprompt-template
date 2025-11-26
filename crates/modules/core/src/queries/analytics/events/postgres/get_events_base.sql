SELECT
    id,
    user_id,
    session_id,
    context_id,
    event_type,
    event_category,
    severity,
    endpoint,
    error_code,
    response_time_ms,
    agent_id,
    task_id,
    message,
    metadata,
    timestamp
FROM analytics_events
WHERE session_id = $1
ORDER BY timestamp ASC