SELECT
    provider,
    model,
    COUNT(*) as request_count,
    SUM(tokens_used) as total_tokens,
    SUM(cost_cents) as total_cost_cents,
    AVG(latency_ms) as avg_latency_ms
FROM ai_requests
WHERE status = 'completed'
    AND created_at >= CURRENT_TIMESTAMP - (CAST(?1 AS INTEGER) || ' days')::INTERVAL
