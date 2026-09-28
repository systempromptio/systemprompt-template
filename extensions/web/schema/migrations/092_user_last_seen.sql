-- When each person was last seen, by any signal the platform records. Twin
-- of schema/47_user_last_seen.sql.

CREATE OR REPLACE VIEW user_last_seen AS
WITH signals AS (
    SELECT user_id, MAX(created_at) AS seen_at, 'request'::text AS source
    FROM ai_requests GROUP BY user_id
    UNION ALL
    SELECT user_id, MAX(last_activity_at), 'session'
    FROM user_sessions WHERE user_id IS NOT NULL GROUP BY user_id
    UNION ALL
    SELECT user_id, MAX(created_at), 'activity'
    FROM user_activity GROUP BY user_id
    UNION ALL
    SELECT user_id, MAX(created_at), 'hook'
    FROM plugin_usage_events GROUP BY user_id
    UNION ALL
    SELECT user_id, MAX(started_at), 'tool'
    FROM mcp_tool_executions GROUP BY user_id
    UNION ALL
    SELECT user_id, MAX(last_heartbeat_at), 'bridge'
    FROM bridge_sessions GROUP BY user_id
)
SELECT DISTINCT ON (user_id)
    user_id,
    seen_at AS last_seen_at,
    source AS last_seen_source
FROM signals
WHERE seen_at IS NOT NULL
ORDER BY user_id, seen_at DESC;
