INSERT INTO ai_requests (
    id,
    request_id,
    user_id,
    session_id,
    trace_id,
    provider,
    model,
    cost_cents,
    latency_ms,
    status,
    request_type,
    image_count
) VALUES (
    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'image', $11
)
