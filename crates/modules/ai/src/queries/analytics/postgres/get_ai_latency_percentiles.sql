-- Get AI request latency percentiles by provider and model (P50, P95, P99)
-- Parameters: $1 = days (e.g., 7 for last 7 days)
SELECT
    provider,
    model,
    PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY latency_ms) as p50_latency_ms,
    PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY latency_ms) as p95_latency_ms,
    PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY latency_ms) as p99_latency_ms,
    AVG(latency_ms) as avg_latency_ms,
    MIN(latency_ms) as min_latency_ms,
    MAX(latency_ms) as max_latency_ms,
    COUNT(*) as request_count
FROM ai_requests
WHERE latency_ms IS NOT NULL
AND created_at >= NOW() - INTERVAL '1 day' * $1
AND status = 'completed'
GROUP BY provider, model
ORDER BY request_count DESC
