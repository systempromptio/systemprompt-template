WITH trace_to_session AS (
    SELECT DISTINCT trace_id, session_id
    FROM ai_requests
    WHERE created_at >= $1 AND created_at < $2
      AND trace_id IS NOT NULL AND session_id IS NOT NULL
),
all_sessions AS (
    SELECT session_id, created_at, NULL::text AS decision, NULL::text AS status
    FROM plugin_usage_events
    WHERE created_at >= $1 AND created_at < $2 AND session_id IS NOT NULL
      AND ($3::TEXT[] IS NULL OR plugin_usage_events.user_id = ANY($3))
    UNION ALL
    SELECT COALESCE(t.session_id, NULLIF(g.session_id, ''), g.trace_id) AS session_id,
           g.created_at, g.decision, NULL::text
    FROM governance_decisions g
    LEFT JOIN trace_to_session t ON t.trace_id = g.trace_id
    WHERE g.created_at >= $1 AND g.created_at < $2
      AND (NULLIF(g.session_id, '') IS NOT NULL OR g.trace_id IS NOT NULL)
      AND ($3::TEXT[] IS NULL OR g.user_id = ANY($3))
    UNION ALL
    SELECT session_id, created_at, NULL::text, status::text
    FROM ai_requests
    WHERE created_at >= $1 AND created_at < $2 AND session_id IS NOT NULL
      AND ($3::TEXT[] IS NULL OR ai_requests.user_id = ANY($3))
),
per_session AS (
    SELECT
        session_id,
        BOOL_OR(decision = 'deny') AS has_deny,
        BOOL_OR(status NOT IN ('ok','success','completed','pending') AND status IS NOT NULL)
          AS has_error
    FROM all_sessions
    GROUP BY session_id
),
-- Percentiles and totals come from the request rows only: a
-- governance-only trace has no latency, and counting it as 0 ms would
-- pin p50 to zero however slow the real traffic was.
active AS (
    SELECT
        session_id,
        COALESCE(SUM(latency_ms), 0)::bigint       AS active_ms,
        COALESCE(SUM(cost_microdollars), 0)::bigint AS cost_microdollars,
        COALESCE(SUM(tokens_used), 0)::bigint       AS tokens
    FROM ai_requests
    WHERE created_at >= $1 AND created_at < $2 AND session_id IS NOT NULL
      AND ($3::TEXT[] IS NULL OR ai_requests.user_id = ANY($3))
    GROUP BY session_id
)
SELECT
    (SELECT COUNT(*) FROM per_session)::bigint                      AS "total_traces!",
    (SELECT COUNT(*) FROM per_session WHERE has_error)::bigint      AS "error_count!",
    (SELECT COUNT(*) FROM per_session WHERE has_deny)::bigint       AS "deny_count!",
    COALESCE(percentile_disc(0.50) WITHIN GROUP (ORDER BY active_ms), 0)::bigint
                                                                    AS "p50!",
    COALESCE(percentile_disc(0.95) WITHIN GROUP (ORDER BY active_ms), 0)::bigint
                                                                    AS "p95!",
    COALESCE(percentile_disc(0.99) WITHIN GROUP (ORDER BY active_ms), 0)::bigint
                                                                    AS "p99!",
    COALESCE(SUM(cost_microdollars), 0)::bigint                     AS "total_cost!",
    COALESCE(SUM(tokens), 0)::bigint                                AS "total_tokens!"
FROM active
