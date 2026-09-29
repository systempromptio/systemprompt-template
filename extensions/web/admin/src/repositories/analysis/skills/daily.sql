-- One skill's day-by-day record inside the window: invocations and people
-- from the hook plane; conversations, tokens, cost and errors from the
-- conversation_facts rows whose harness session invoked it that day.
WITH days AS (
    SELECT generate_series(date_trunc('day', $1::timestamptz), date_trunc('day', $2::timestamptz), interval '1 day')::date AS day
), events AS (
    SELECT e.user_id, e.session_id, date_trunc('day', e.invoked_at)::date AS day
    FROM analysis_skill_version_events e
    WHERE e.skill = $3 AND e.invoked_at >= $1 AND e.invoked_at < $2
      AND ($4::text[] IS NULL OR e.user_id = ANY($4))
), per_day AS (
    SELECT e.day, COUNT(*)::bigint AS invocations, COUNT(DISTINCT e.user_id)::bigint AS users,
           COUNT(DISTINCT e.session_id)::bigint AS sessions
    FROM events e GROUP BY e.day
), facts AS (
    SELECT s.day,
           COUNT(DISTINCT f.context_id)::bigint AS conversations,
           COALESCE(SUM(f.input_tokens + f.output_tokens), 0)::bigint AS tokens,
           COALESCE(SUM(f.cost_microdollars), 0)::bigint AS cost_microdollars,
           COALESCE(SUM(f.error_count + f.gov_deny), 0)::bigint AS errors,
           percentile_cont(0.95) WITHIN GROUP (ORDER BY f.p95_latency_ms)::float8 AS p95_latency_ms,
           AVG(a.completion)::float8 AS completion_avg
    FROM (SELECT DISTINCT day, session_id FROM events) s
    JOIN conversation_facts f ON f.client_session_id = s.session_id
    LEFT JOIN conversation_analyses a ON a.context_id = f.context_id
    GROUP BY s.day
)
SELECT d.day AS "day!",
       COALESCE(p.invocations, 0)::bigint AS "invocations!",
       COALESCE(p.users, 0)::bigint AS "users!",
       COALESCE(p.sessions, 0)::bigint AS "sessions!",
       COALESCE(f.conversations, 0)::bigint AS "conversations!",
       COALESCE(f.tokens, 0)::bigint AS "tokens!",
       COALESCE(f.cost_microdollars, 0)::bigint AS "cost_microdollars!",
       COALESCE(f.errors, 0)::bigint AS "errors!",
       f.p95_latency_ms, f.completion_avg
FROM days d
LEFT JOIN per_day p ON p.day = d.day
LEFT JOIN facts f ON f.day = d.day
ORDER BY d.day
