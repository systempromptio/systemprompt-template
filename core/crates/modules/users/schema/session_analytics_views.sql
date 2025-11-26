-- Analytics views for user sessions (users module only)
-- These views only reference user_sessions table
-- Cross-module views (with user_contexts, task_messages) are in agent module

-- View: Daily conversion metrics
DROP VIEW IF EXISTS v_daily_conversions CASCADE;
CREATE VIEW v_daily_conversions AS
SELECT
    DATE(started_at) as date,
    COUNT(CASE WHEN user_type = 'anon' THEN 1 END) as anonymous_sessions,
    COUNT(CASE WHEN user_type = 'registered' AND converted_at IS NOT NULL THEN 1 END) as converted_sessions,
    ROUND(
        CAST(COUNT(CASE WHEN user_type = 'registered' AND converted_at IS NOT NULL THEN 1 END) AS NUMERIC) * 100.0 /
        NULLIF(COUNT(CASE WHEN user_type = 'anon' OR (user_type = 'registered' AND converted_at IS NOT NULL) THEN 1 END), 0),
        2
    ) as conversion_rate_pct
FROM user_sessions
WHERE started_at >= CURRENT_DATE - INTERVAL '30 days'
GROUP BY DATE(started_at)
ORDER BY date DESC;

-- View: Time to conversion metrics
DROP VIEW IF EXISTS v_time_to_conversion CASCADE;
CREATE VIEW v_time_to_conversion AS
SELECT
    session_id,
    user_id,
    started_at,
    converted_at,
    ROUND(EXTRACT(EPOCH FROM (converted_at - started_at))/60.0, 2) as minutes_to_convert,
    CASE
        WHEN EXTRACT(EPOCH FROM (converted_at - started_at))/60.0 < 5 THEN 'under_5_min'
        WHEN EXTRACT(EPOCH FROM (converted_at - started_at))/60.0 < 15 THEN '5_to_15_min'
        WHEN EXTRACT(EPOCH FROM (converted_at - started_at))/60.0 < 60 THEN '15_to_60_min'
        WHEN EXTRACT(EPOCH FROM (converted_at - started_at)) < 86400 THEN '1_to_24_hours'
        ELSE 'over_24_hours'
    END as time_bucket
FROM user_sessions
WHERE user_type = 'registered'
  AND converted_at IS NOT NULL
ORDER BY converted_at DESC;
