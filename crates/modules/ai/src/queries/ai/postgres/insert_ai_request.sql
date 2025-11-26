INSERT INTO ai_requests (
    id,
    request_id,
    user_id,
    session_id,
    task_id,
    context_id,
    trace_id,
    provider,
    model,
    temperature,
    top_p,
    max_tokens,
    stop_sequences,
    tokens_used,
    input_tokens,
    output_tokens,
    cache_hit,
    cache_read_tokens,
    cache_creation_tokens,
    is_streaming,
    cost_cents,
    latency_ms,
    status,
    error_message
) VALUES (
    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24
)
