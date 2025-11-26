SELECT
    id,
    request_id,
    user_id,
    session_id,
    task_id,
    context_id,
    trace_id,
    provider,
    model,
    messages,
    tool_calls,
    response,
    sampling_metadata,
    tokens_used,
    cost_cents,
    latency_ms,
    status,
    error_message,
    created_at,
    completed_at
FROM ai_requests
WHERE request_id = $1
