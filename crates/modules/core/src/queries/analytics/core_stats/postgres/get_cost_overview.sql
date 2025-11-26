SELECT
    COALESCE(SUM(total_ai_cost_cents), 0) as total_cost_cents,
    COUNT(DISTINCT COALESCE(user_id, fingerprint_hash)) as unique_users,
    COUNT(*) as total_sessions,
    COALESCE(AVG(total_ai_cost_cents), 0.0) as avg_cost_per_session_cents
FROM user_sessions
WHERE started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
    AND total_ai_cost_cents > 0
