-- Cross-module analytics views for user session conversion
-- These views join user_sessions (users module) with user_contexts and task_messages (agent module)
-- Located in agent module because it loads after both users and agent base tables

-- View: Pre-conversion engagement stats
DROP VIEW IF EXISTS v_preconversion_engagement CASCADE;
CREATE VIEW v_preconversion_engagement AS
SELECT
    s.session_id,
    s.user_id,
    s.started_at,
    s.converted_at,
    COUNT(DISTINCT c.context_id) as contexts_created,
    COUNT(DISTINCT tm.message_id) as messages_sent,
    ROUND(EXTRACT(EPOCH FROM (s.converted_at - s.started_at))/60.0, 2) as minutes_to_convert
FROM user_sessions s
LEFT JOIN user_contexts c ON s.session_id = c.session_id
LEFT JOIN task_messages tm ON c.context_id = tm.context_id
WHERE s.user_type = 'registered'
  AND s.converted_at IS NOT NULL
GROUP BY s.session_id, s.user_id, s.started_at, s.converted_at
ORDER BY s.converted_at DESC;

-- View: Conversion funnel stage analysis
DROP VIEW IF EXISTS v_conversion_funnel CASCADE;
CREATE VIEW v_conversion_funnel AS
SELECT
    'Total Anonymous Sessions' as stage,
    COUNT(*) as count,
    100.0 as percent_of_start
FROM user_sessions
WHERE user_type = 'anon' OR (user_type = 'registered' AND converted_at IS NOT NULL)
UNION ALL
SELECT
    'Anonymous with Contexts' as stage,
    COUNT(DISTINCT s.session_id) as count,
    ROUND(
        CAST(COUNT(DISTINCT s.session_id) AS NUMERIC) * 100.0 /
        NULLIF((SELECT COUNT(*) FROM user_sessions WHERE user_type = 'anon' OR (user_type = 'registered' AND converted_at IS NOT NULL)), 0),
        2
    ) as percent_of_start
FROM user_sessions s
INNER JOIN user_contexts c ON s.session_id = c.session_id
WHERE s.user_type = 'anon' OR (s.user_type = 'registered' AND s.converted_at IS NOT NULL)
UNION ALL
SELECT
    'Anonymous with Messages' as stage,
    COUNT(DISTINCT s.session_id) as count,
    ROUND(
        CAST(COUNT(DISTINCT s.session_id) AS NUMERIC) * 100.0 /
        NULLIF((SELECT COUNT(*) FROM user_sessions WHERE user_type = 'anon' OR (user_type = 'registered' AND converted_at IS NOT NULL)), 0),
        2
    ) as percent_of_start
FROM user_sessions s
INNER JOIN user_contexts c ON s.session_id = c.session_id
INNER JOIN task_messages tm ON c.context_id = tm.context_id
WHERE s.user_type = 'anon' OR (s.user_type = 'registered' AND s.converted_at IS NOT NULL)
UNION ALL
SELECT
    'Converted to Registered' as stage,
    COUNT(*) as count,
    ROUND(
        CAST(COUNT(*) AS NUMERIC) * 100.0 /
        NULLIF((SELECT COUNT(*) FROM user_sessions WHERE user_type = 'anon' OR (user_type = 'registered' AND converted_at IS NOT NULL)), 0),
        2
    ) as percent_of_start
FROM user_sessions
WHERE user_type = 'registered' AND converted_at IS NOT NULL;

-- View: Active anonymous sessions summary
DROP VIEW IF EXISTS v_active_anonymous_sessions CASCADE;
CREATE VIEW v_active_anonymous_sessions AS
SELECT
    s.session_id,
    s.user_id,
    s.started_at,
    s.last_activity_at,
    s.expires_at,
    COUNT(DISTINCT c.context_id) as context_count,
    COUNT(DISTINCT tm.message_id) as message_count,
    ROUND(EXTRACT(EPOCH FROM (NOW() - s.started_at))/60.0, 2) as session_age_minutes,
    ROUND(EXTRACT(EPOCH FROM (s.expires_at - NOW()))/60.0, 2) as minutes_until_expiry
FROM user_sessions s
LEFT JOIN user_contexts c ON s.session_id = c.session_id
LEFT JOIN task_messages tm ON c.context_id = tm.context_id
WHERE s.user_type = 'anon'
  AND s.expires_at > NOW()
GROUP BY s.session_id, s.user_id, s.started_at, s.last_activity_at, s.expires_at
ORDER BY s.last_activity_at DESC;
