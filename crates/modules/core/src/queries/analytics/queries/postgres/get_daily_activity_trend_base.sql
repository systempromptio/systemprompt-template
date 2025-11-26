SELECT
    DATE(s.started_at) as activity_date,
    COUNT(*) as sessions,
    COUNT(DISTINCT s.user_id) as unique_users,
    SUM(s.request_count) as total_requests,
    SUM(s.ai_request_count) as ai_requests,
    SUM(s.total_tokens_used) as tokens_used,
    SUM(s.total_ai_cost_cents) as cost_cents,
    AVG(s.avg_response_time_ms) as avg_response_time
FROM user_sessions s
WHERE s.started_at >= datetime('now', '-' || $1 || ' days')