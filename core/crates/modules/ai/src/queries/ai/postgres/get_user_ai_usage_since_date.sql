SELECT
    provider,
    model,
    COUNT(*) as request_count,
    SUM(tokens_used) as total_tokens,
    SUM(cost_cents) as total_cost_cents,
    AVG(latency_ms) as avg_latency_ms,
    CAST(created_at AS DATE) as usage_date
FROM ai_requests
WHERE user_id = $1 AND status = 'completed'
  AND created_at >= $2
GROUP BY provider, model, CAST(created_at AS DATE)
ORDER BY created_at DESC
