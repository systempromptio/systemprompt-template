WITH events AS (
    SELECT e.skill, e.user_id, e.session_id, e.attribution_status
    FROM analysis_skill_version_events e
    WHERE e.invoked_at >= $1 AND e.invoked_at < $2
      AND e.skill IS NOT NULL
      AND ($3::TEXT[] IS NULL OR e.user_id = ANY($3))
      AND ($4::TEXT IS NULL OR e.user_id = $4)
),
conversations AS (
    SELECT DISTINCT user_id, session_id FROM events
),
requests AS (
    SELECT r.id, r.cost_microdollars
    FROM conversations c
    JOIN ai_requests r ON r.user_id = c.user_id AND r.client_session_id = c.session_id
                       AND NOT r.synthetic AND r.request_kind = 'turn'
)
SELECT
    (SELECT COUNT(*) FROM events)::BIGINT AS "invocations!",
    (SELECT COUNT(*) FROM events WHERE attribution_status = 'verified')::BIGINT AS "attributed!",
    (SELECT COUNT(DISTINCT skill) FROM events)::BIGINT AS "skills!",
    (SELECT COUNT(DISTINCT user_id) FROM events)::BIGINT AS "users!",
    (SELECT COUNT(*) FROM conversations)::BIGINT AS "conversations!",
    (SELECT COUNT(*) FROM requests)::BIGINT AS "requests!",
    (SELECT COALESCE(SUM(cost_microdollars), 0) FROM requests)::BIGINT AS "cost!"
