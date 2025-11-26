-- List all sessions for a specific user
--
-- Parameters:
--   $1: user_id (TEXT) - The user's ID to retrieve sessions for
--
-- Returns: All sessions for the user with session metrics
--
-- Usage: Used by admin tools to view a user's session history
SELECT
    s.session_id,
    s.user_id,
    s.started_at,
    s.last_activity_at,
    s.ended_at,
    s.duration_seconds,
    s.user_type,
    s.client_id,
    s.client_type,
    s.request_count,
    s.avg_response_time_ms,
    s.success_rate,
    s.error_count,
    s.task_count,
    s.message_count,
    s.ai_request_count,
    s.total_tokens_used,
    s.total_ai_cost_cents,
    s.ip_address,
    s.user_agent,
    s.device_type,
    s.browser,
    s.os,
    s.country,
    s.region,
    s.city
FROM user_sessions s
WHERE s.user_id = $1
ORDER BY s.started_at DESC
LIMIT 50
