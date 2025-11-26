-- Get cache hit rates and efficiency metrics by provider and model
-- Parameters: $1 = days (e.g., 7 for last 7 days)
SELECT
    provider,
    model,
    COUNT(*) as total_requests,
    SUM(CASE WHEN cache_hit = true THEN 1 ELSE 0 END) as cache_hits,
    CAST(SUM(CASE WHEN cache_hit = true THEN 1 ELSE 0 END) AS FLOAT) / NULLIF(COUNT(*), 0) * 100 as cache_hit_rate_percent,
    SUM(COALESCE(cache_read_tokens, 0)) as total_cache_read_tokens,
    SUM(COALESCE(cache_creation_tokens, 0)) as total_cache_creation_tokens,
    AVG(CASE WHEN cache_hit = true THEN latency_ms END) as avg_latency_with_cache_ms,
    AVG(CASE WHEN cache_hit = false THEN latency_ms END) as avg_latency_without_cache_ms
FROM ai_requests
WHERE created_at >= NOW() - INTERVAL '1 day' * $1
AND status = 'completed'
GROUP BY provider, model
HAVING COUNT(*) >= 10  -- Only show models with at least 10 requests
ORDER BY cache_hit_rate_percent DESC NULLS LAST
