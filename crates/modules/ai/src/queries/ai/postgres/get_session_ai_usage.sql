SELECT
    COUNT(*) as request_count,
    SUM(tokens_used) as total_tokens,
    SUM(cost_cents) as total_cost_cents,
    AVG(latency_ms) as avg_latency_ms,
    MIN(created_at) as first_request,
    MAX(created_at) as last_request
FROM ai_requests
WHERE session_id = $1 AND status = 'completed'
