INSERT INTO analytics_events (
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
    metadata
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8::INTEGER, $9::INTEGER, $10, $11, $12, $13)