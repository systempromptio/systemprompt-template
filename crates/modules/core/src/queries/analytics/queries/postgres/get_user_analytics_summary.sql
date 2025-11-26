SELECT
    COUNT(*) as total_sessions,
    SUM(request_count) as total_requests,
    SUM(ai_request_count) as total_ai_requests,
    SUM(total_tokens_used) as total_tokens,
    SUM(total_ai_cost_cents) as total_cost_cents,
    AVG(avg_response_time_ms) as avg_response_time,
    SUM(task_count) as total_tasks,
    SUM(message_count) as total_messages,
    COUNT(DISTINCT DATE(started_at)) as active_days,
    SUM(error_count) as total_errors,
    AVG(success_rate) as avg_success_rate
FROM user_sessions
WHERE user_id = $1
AND started_at >= datetime('now', '-' || $2 || ' days')