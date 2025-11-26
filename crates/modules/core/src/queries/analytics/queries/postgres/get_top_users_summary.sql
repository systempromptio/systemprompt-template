SELECT
    COALESCE(user_id, '(anonymous)') as user_id,
    COUNT(*) as total_sessions,
    SUM(request_count) as total_requests,
    SUM(ai_request_count) as total_ai_requests,
    SUM(total_tokens_used) as total_tokens,
    SUM(total_ai_cost_cents) as total_cost_cents,
    AVG(avg_response_time_ms) as avg_response_time,
    COUNT(DISTINCT DATE(started_at)) as active_days,
    SUM(error_count) as total_errors,
    AVG(success_rate) as avg_success_rate
FROM user_sessions
WHERE started_at >= datetime('now', '-' || $1 || ' days')
GROUP BY user_id
ORDER BY total_requests DESC
LIMIT $2
