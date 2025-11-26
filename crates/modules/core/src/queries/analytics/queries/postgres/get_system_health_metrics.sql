SELECT
    (SELECT COUNT(*) FROM user_sessions WHERE ended_at IS NULL) as active_sessions,
    (SELECT SUM(request_count) FROM user_sessions WHERE last_activity_at >= datetime('now', '-' || $1 || ' hours')) as total_requests,
    (SELECT AVG(avg_response_time_ms) FROM user_sessions WHERE last_activity_at >= datetime('now', '-' || $2 || ' hours')) as system_avg_response_time,
    (SELECT SUM(error_count) FROM user_sessions WHERE last_activity_at >= datetime('now', '-' || $3 || ' hours')) as total_errors,
    (SELECT AVG(success_rate) FROM user_sessions WHERE last_activity_at >= datetime('now', '-' || $4 || ' hours')) as system_success_rate,
    (SELECT COUNT(DISTINCT user_id) FROM user_sessions WHERE last_activity_at >= datetime('now', '-' || $5 || ' hours')) as active_users,
    (SELECT COUNT(*) FROM logs WHERE level = 'critical' AND timestamp >= datetime('now', '-' || $6 || ' hours')) as critical_events,
    (SELECT COUNT(*) FROM logs WHERE level = 'error' AND timestamp >= datetime('now', '-' || $7 || ' hours')) as error_events