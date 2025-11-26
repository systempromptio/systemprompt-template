SELECT
    COUNT(*) as total_sessions,
    COUNT(CASE WHEN request_count = 1 THEN 1 END) as bounce_sessions,
    COUNT(CASE WHEN request_count > 1 THEN 1 END) as active_sessions,
    AVG(request_count) as avg_requests_per_session,
    AVG(CASE WHEN ended_at IS NOT NULL THEN duration_seconds END) as avg_session_duration,
    COUNT(CASE WHEN fingerprint_hash IS NOT NULL THEN 1 END) as fingerprinted_sessions,
    COUNT(CASE WHEN user_id IS NOT NULL THEN 1 END) as authenticated_sessions
FROM user_sessions
WHERE started_at >= datetime('now', '-' || $1 || ' days')
