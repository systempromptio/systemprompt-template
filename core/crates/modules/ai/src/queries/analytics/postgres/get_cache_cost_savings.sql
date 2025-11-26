-- Calculate cost savings from prompt caching
-- Parameters: $1 = days (e.g., 7 for last 7 days)
SELECT
    provider,
    model,
    COUNT(*) as total_requests,
    SUM(CASE WHEN cache_hit = true THEN 1 ELSE 0 END) as cached_requests,
    SUM(cost_cents) as total_cost_cents,
    -- Estimate cost savings: cache read tokens are typically 10% of normal token cost
    SUM(CASE
        WHEN cache_hit = true THEN
            (COALESCE(cache_read_tokens, 0) * 0.9 * cost_cents) / NULLIF(COALESCE(input_tokens, tokens_used, 0), 0)
        ELSE 0
    END) as estimated_savings_cents,
    -- Cost breakdown
    SUM(CASE WHEN cache_hit = true THEN cost_cents ELSE 0 END) as cost_with_cache_cents,
    SUM(CASE WHEN cache_hit = false THEN cost_cents ELSE 0 END) as cost_without_cache_cents
FROM ai_requests
WHERE created_at >= NOW() - INTERVAL '1 day' * $1
AND status = 'completed'
AND cost_cents > 0
GROUP BY provider, model
HAVING SUM(CASE WHEN cache_hit = true THEN 1 ELSE 0 END) > 0  -- Only show models with cache hits
ORDER BY estimated_savings_cents DESC
