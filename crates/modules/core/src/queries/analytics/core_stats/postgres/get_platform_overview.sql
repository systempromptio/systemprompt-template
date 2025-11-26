SELECT
    (SELECT COUNT(*) FROM users WHERE status != 'temporary') as total_users,
    (SELECT COUNT(DISTINCT user_id)
     FROM user_sessions
     WHERE last_activity_at >= NOW() - ($1::INTEGER || ' hours')::INTERVAL
     AND (request_count > 0 OR task_count > 0 OR ai_request_count > 0)) as active_users,
    (SELECT COUNT(*)
     FROM user_sessions
     WHERE last_activity_at >= NOW() - ($2::INTEGER || ' hours')::INTERVAL
     AND ended_at IS NULL
     AND (request_count > 0 OR task_count > 0 OR ai_request_count > 0)) as active_sessions,
    (SELECT COUNT(*)
     FROM ai_requests
     WHERE created_at >= NOW() - ($3::INTEGER || ' hours')::INTERVAL) as ai_requests_24h,
    (SELECT COALESCE(SUM(cost_cents), 0)
     FROM ai_requests
     WHERE created_at >= NOW() - ($4::INTEGER || ' hours')::INTERVAL) as cost_cents_24h,
    (SELECT COALESCE(SUM(cost_cents), 0)
     FROM ai_requests
     WHERE created_at >= NOW() - INTERVAL '7 days') as cost_cents_7d,
    (SELECT AVG(avg_response_time_ms)
     FROM user_sessions
     WHERE last_activity_at >= NOW() - ($5::INTEGER || ' hours')::INTERVAL) as avg_response_time_ms,
    (SELECT AVG(success_rate)
     FROM user_sessions
     WHERE last_activity_at >= NOW() - ($6::INTEGER || ' hours')::INTERVAL) as success_rate,
    (SELECT SUM(error_count)
     FROM user_sessions
     WHERE last_activity_at >= NOW() - ($7::INTEGER || ' hours')::INTERVAL) as total_errors;
