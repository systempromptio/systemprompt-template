WITH events AS (
    SELECT e.skill, e.user_id, e.session_id, e.source, e.resource_id, e.attribution_status
    FROM analysis_skill_version_events e
    WHERE e.invoked_at >= $1 AND e.invoked_at < $2
      AND e.skill IS NOT NULL
      AND ($3::TEXT[] IS NULL OR e.user_id = ANY($3))
      AND ($4::TEXT IS NULL OR e.user_id = $4)
),
conversations AS (
    SELECT DISTINCT skill, user_id, session_id FROM events
),
spend AS (
    SELECT c.skill,
           COUNT(r.id)::BIGINT AS requests,
           COUNT(r.id) FILTER (WHERE r.cost_microdollars > 0)::BIGINT AS priced_requests,
           COALESCE(SUM(r.cost_microdollars), 0)::BIGINT AS cost,
           COALESCE(SUM(COALESCE(r.input_tokens, 0) + COALESCE(r.output_tokens, 0)), 0)::BIGINT AS tokens
    FROM conversations c
    JOIN ai_requests r ON r.user_id = c.user_id AND r.client_session_id = c.session_id
                       AND NOT r.synthetic AND r.request_kind = 'turn'
    GROUP BY c.skill
)
SELECT
    ev.skill AS "skill!",
    MAX(ev.resource_id) FILTER (WHERE ev.attribution_status = 'verified') AS resource_id,
    COUNT(*)::BIGINT AS "invocations!",
    COUNT(*) FILTER (WHERE ev.source = 'slash')::BIGINT AS "slash!",
    COUNT(*) FILTER (WHERE ev.source = 'tool')::BIGINT AS "tool!",
    COUNT(*) FILTER (WHERE ev.attribution_status = 'verified')::BIGINT AS "attributed!",
    COUNT(DISTINCT ev.user_id)::BIGINT AS "distinct_users!",
    COUNT(DISTINCT ev.session_id)::BIGINT AS "conversations!",
    COALESCE(MAX(s.requests), 0)::BIGINT AS "requests!",
    COALESCE(MAX(s.priced_requests), 0)::BIGINT AS "priced_requests!",
    COALESCE(MAX(s.cost), 0)::BIGINT AS "cost!",
    COALESCE(MAX(s.tokens), 0)::BIGINT AS "tokens!",
    COUNT(*) OVER ()::BIGINT AS "total!"
FROM events ev
LEFT JOIN spend s ON s.skill = ev.skill
GROUP BY ev.skill
ORDER BY COUNT(*) DESC, ev.skill
LIMIT $5 OFFSET $6
