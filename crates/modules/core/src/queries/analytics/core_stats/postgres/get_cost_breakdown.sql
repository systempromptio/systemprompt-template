SELECT
    provider,
    model,
    COUNT(*) as request_count,
    COALESCE(SUM(tokens_used), 0) as total_tokens,
    COALESCE(SUM(cost_cents), 0) as cost_cents,
    ROUND(AVG(latency_ms), 2) as avg_latency_ms,
    COUNT(DISTINCT COALESCE(user_id, session_id)) as unique_users,
    COUNT(DISTINCT session_id) as unique_sessions
FROM ai_requests
WHERE created_at >= datetime('now', '-' || $1 || ' days')
    AND provider IS NOT NULL
GROUP BY provider, model
ORDER BY cost_cents DESC;
