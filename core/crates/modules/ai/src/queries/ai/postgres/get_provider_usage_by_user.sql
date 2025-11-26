SELECT
    provider,
    model,
    COUNT(*) as request_count,
    SUM(tokens_used) as total_tokens,
    SUM(cost_cents) as total_cost_cents,
    AVG(latency_ms) as avg_latency_ms
FROM ai_requests
WHERE status = 'completed'
    AND created_at >= NOW() - INTERVAL '1 day' * $1
    AND user_id = $2
GROUP BY provider, model
ORDER BY total_cost_cents DESC
