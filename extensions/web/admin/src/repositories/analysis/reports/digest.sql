-- The deterministic digest an AI report is written from: the conversation
-- record narrowed to a scope and a window, summarised into totals, the top
-- skills, models and people, the best and worst judged conversations, the
-- cost outliers, the denial clusters and the intent and client mix. Every
-- figure is a count, sum or mean over conversation_facts / conversation_analyses
-- rows; no transcript is read. Every narrowing is a bound parameter.
WITH f AS MATERIALIZED (
    SELECT f.*, u.display_name,
           f.input_tokens + f.output_tokens AS total_tokens,
           a.title AS judge_title, a.category, a.outcome, a.completion, a.summary,
           COALESCE(a.skills_used, '{}'::text[]) AS skills_used,
           conversation_title(f.context_id, f.client_session_id) AS title
    FROM conversation_facts f
    LEFT JOIN conversation_analyses a ON a.context_id = f.context_id
    LEFT JOIN users u ON u.id = f.user_id
    WHERE f.last_at >= $1 AND f.last_at < $2
      AND ($3::text[] IS NULL OR f.user_id = ANY($3))
      AND ($4::text IS NULL OR EXISTS (
            SELECT 1 FROM unnest(f.skills) s(skill)
            JOIN service_owned_ids o ON o.kind = 'plugin' AND o.id = split_part(s.skill, ':', 1)
            WHERE o.marketplace_id = $4))
      AND ($5::text IS NULL OR $5 = ANY(f.skills) OR $5 = ANY(COALESCE(a.skills_used, '{}')))
      AND ($6::text IS NULL OR f.user_id = $6)
      AND ($7::text IS NULL OR f.model = $7 OR $7 = ANY(f.models))
      AND ($8::text IS NULL OR f.client_kind = $8)
      AND ($9::text IS NULL OR a.category = $9)
      AND ($10::text IS NULL OR a.outcome = $10)
), totals AS (
    SELECT COUNT(*)::bigint AS conversations, COUNT(DISTINCT user_id)::bigint AS people,
           COALESCE(SUM(turn_count), 0)::bigint AS turns,
           COALESCE(SUM(total_tokens), 0)::bigint AS tokens,
           COALESCE(SUM(cost_microdollars), 0)::bigint AS cost_microdollars,
           COALESCE(SUM(error_count), 0)::bigint AS errors,
           COALESCE(SUM(gov_deny), 0)::bigint AS denied,
           COALESCE(SUM(safety_findings), 0)::bigint AS safety_findings,
           COALESCE(SUM(artifact_count), 0)::bigint AS artifacts,
           COALESCE(SUM(tool_calls_intended), 0)::bigint AS tool_calls,
           COUNT(*) FILTER (WHERE completion IS NOT NULL)::bigint AS judged,
           AVG(completion)::float8 AS completion_avg,
           COUNT(*) FILTER (WHERE outcome = 'achieved')::bigint AS achieved,
           COUNT(*) FILTER (WHERE outcome = 'partial')::bigint AS partial,
           COUNT(*) FILTER (WHERE outcome = 'abandoned')::bigint AS abandoned,
           COUNT(*) FILTER (WHERE outcome = 'unclear')::bigint AS unclear
    FROM f
), skills AS (
    SELECT s.skill,
           (SELECT COUNT(*)::bigint FROM analysis_skill_version_events e
             WHERE e.skill = s.skill AND e.invoked_at >= $1 AND e.invoked_at < $2
               AND ($3::text[] IS NULL OR e.user_id = ANY($3))) AS invocations,
           COUNT(*)::bigint AS conversations,
           COALESCE(SUM(s.cost_microdollars), 0)::bigint AS cost_microdollars,
           AVG(s.completion)::float8 AS completion_avg,
           COALESCE(SUM(s.error_count + s.gov_deny), 0)::bigint AS errors
    FROM (SELECT unnest(f.skills) AS skill, f.cost_microdollars, f.completion, f.error_count, f.gov_deny FROM f) s
    GROUP BY s.skill ORDER BY invocations DESC, conversations DESC, s.skill LIMIT 10
), models AS (
    SELECT COALESCE(f.model, 'unknown') AS model, COUNT(*)::bigint AS conversations,
           COALESCE(SUM(f.request_count), 0)::bigint AS requests,
           COALESCE(SUM(f.cost_microdollars), 0)::bigint AS cost_microdollars,
           COALESCE(SUM(f.error_count), 0)::bigint AS errors,
           AVG(f.completion)::float8 AS completion_avg
    FROM f GROUP BY 1 ORDER BY cost_microdollars DESC, conversations DESC LIMIT 10
), people AS (
    SELECT f.user_id, MAX(f.display_name) AS display_name, COUNT(*)::bigint AS conversations,
           COALESCE(SUM(f.cost_microdollars), 0)::bigint AS cost_microdollars,
           AVG(f.completion)::float8 AS completion_avg,
           COALESCE(SUM(f.error_count + f.gov_deny), 0)::bigint AS errors
    FROM f GROUP BY f.user_id ORDER BY cost_microdollars DESC LIMIT 10
), judged AS (
    SELECT f.context_id, f.title, f.judge_title, f.completion, f.outcome, f.category,
           f.cost_microdollars, f.turn_count AS turns, LEFT(f.summary, 200) AS summary
    FROM f WHERE f.completion IS NOT NULL
), worst AS (
    SELECT * FROM judged ORDER BY completion ASC, cost_microdollars DESC LIMIT 8
), best AS (
    SELECT * FROM judged ORDER BY completion DESC, cost_microdollars DESC LIMIT 5
), outliers AS (
    SELECT f.context_id, f.title, f.cost_microdollars, f.turn_count AS turns, f.total_tokens AS tokens,
           f.completion, f.model
    FROM f ORDER BY f.cost_microdollars DESC LIMIT 8
), denials AS (
    SELECT d.tool_name, COUNT(*)::bigint AS denied, COUNT(DISTINCT d.user_id)::bigint AS people
    FROM governance_decisions d
    WHERE d.decision = 'deny' AND d.created_at >= $1 AND d.created_at < $2
      AND ($3::text[] IS NULL OR d.user_id = ANY($3))
      AND ($6::text IS NULL OR d.user_id = $6)
    GROUP BY d.tool_name ORDER BY denied DESC LIMIT 10
), intents AS (
    SELECT COALESCE(f.category, 'unjudged') AS category, COUNT(*)::bigint AS conversations,
           AVG(f.completion)::float8 AS completion_avg
    FROM f GROUP BY 1 ORDER BY conversations DESC
), clients AS (
    SELECT f.client_kind, COUNT(*)::bigint AS conversations,
           COALESCE(SUM(f.cost_microdollars), 0)::bigint AS cost_microdollars
    FROM f GROUP BY 1 ORDER BY conversations DESC
)
SELECT jsonb_build_object(
    'window_start', $1::timestamptz, 'window_end', $2::timestamptz,
    'totals', (SELECT to_jsonb(t) FROM totals t),
    'skills', COALESCE((SELECT jsonb_agg(to_jsonb(s)) FROM skills s), '[]'::jsonb),
    'models', COALESCE((SELECT jsonb_agg(to_jsonb(m)) FROM models m), '[]'::jsonb),
    'people', COALESCE((SELECT jsonb_agg(to_jsonb(p)) FROM people p), '[]'::jsonb),
    'worst', COALESCE((SELECT jsonb_agg(to_jsonb(w)) FROM worst w), '[]'::jsonb),
    'best', COALESCE((SELECT jsonb_agg(to_jsonb(b)) FROM best b), '[]'::jsonb),
    'outliers', COALESCE((SELECT jsonb_agg(to_jsonb(o)) FROM outliers o), '[]'::jsonb),
    'denials', COALESCE((SELECT jsonb_agg(to_jsonb(d)) FROM denials d), '[]'::jsonb),
    'intents', COALESCE((SELECT jsonb_agg(to_jsonb(i)) FROM intents i), '[]'::jsonb),
    'clients', COALESCE((SELECT jsonb_agg(to_jsonb(c)) FROM clients c), '[]'::jsonb)
) AS "payload!: Json<ReportDigest>"
