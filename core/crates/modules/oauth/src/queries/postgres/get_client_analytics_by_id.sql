SELECT
    client_id,
    COUNT(DISTINCT session_id) as session_count,
    COUNT(DISTINCT user_id) as unique_users,
    COUNT(*) as total_requests,
    COALESCE(SUM(tokens_used), 0) as total_tokens,
    COALESCE(SUM(cost_cents), 0) as total_cost_cents,
    COALESCE(AVG(EXTRACT(EPOCH FROM (ended_at - created_at))), 0.0) as avg_session_duration_seconds,
    COALESCE(AVG(response_time_ms), 0.0) as avg_response_time_ms,
    MIN(created_at) as first_seen,
    MAX(created_at) as last_seen
FROM oauth_sessions
WHERE client_id = $1
GROUP BY client_id
