SELECT
    user_id,
    COUNT(*) as request_count,
    SUM(tokens_used) as total_tokens,
    SUM(cost_cents) as total_cost_cents,
    AVG(latency_ms) as avg_latency_ms
FROM ai_requests
WHERE user_id IS NOT NULL
    AND status = 'completed'
    AND created_at >= CURRENT_TIMESTAMP - (CAST(?1 AS INTEGER) || ' days')::INTERVAL
GROUP BY user_id
ORDER BY total_cost_cents DESC
