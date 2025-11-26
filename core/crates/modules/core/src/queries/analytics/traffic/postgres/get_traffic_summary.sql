SELECT
    COUNT(DISTINCT session_id) AS total_sessions,
    SUM(COALESCE(request_count, 0)) AS total_requests,
    COUNT(DISTINCT COALESCE(user_id, fingerprint_hash)) AS unique_users,
    AVG(
        CAST(
            CASE
                WHEN ended_at IS NOT NULL THEN COALESCE(duration_seconds, EXTRACT(EPOCH FROM (ended_at - started_at))::INTEGER)
                ELSE EXTRACT(EPOCH FROM (last_activity_at - started_at))::INTEGER
            END AS FLOAT
        )
    ) AS avg_session_duration_secs,
    CASE
        WHEN COUNT(DISTINCT session_id) > 0
        THEN SUM(COALESCE(request_count, 0))::DOUBLE PRECISION / COUNT(DISTINCT session_id)
        ELSE 0
    END AS avg_requests_per_session,
    SUM(COALESCE(total_ai_cost_cents, 0)) AS total_cost_cents
FROM user_sessions
WHERE started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
  AND is_bot = FALSE
  AND is_scanner = FALSE
  AND request_count > 0
