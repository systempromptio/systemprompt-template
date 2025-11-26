-- Get latency percentiles by endpoint (P50, P95, P99)
-- Parameters: $1 = days (e.g., 7 for last 7 days)
SELECT
    endpoint,
    PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY response_time_ms) as p50_latency_ms,
    PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY response_time_ms) as p95_latency_ms,
    PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY response_time_ms) as p99_latency_ms,
    AVG(response_time_ms) as avg_latency_ms,
    COUNT(*) as request_count
FROM analytics_events
WHERE response_time_ms IS NOT NULL
AND endpoint IS NOT NULL
AND timestamp >= NOW() - INTERVAL '1 day' * $1
GROUP BY endpoint
HAVING COUNT(*) >= 10  -- Only show endpoints with at least 10 requests
ORDER BY request_count DESC
LIMIT 50
