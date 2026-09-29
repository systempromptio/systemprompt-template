-- One statement for the conversations page over the conversation_facts
-- rollup: the filtered set once, then its totals, its time series, one
-- breakdown dimension, the facet lists and one page of rows. Every row is a
-- conversation whether or not the judge has seen it; the judge's row is a
-- LEFT JOIN. Filtering, paging and sorting are bound parameters chosen by
-- CASE arms, never interpolated text. The series is joined onto a
-- generate_series of buckets so a thirty-day window always yields thirty
-- points, zeros included — two busy days must not draw as one slope.
-- A conversation with no turn (title and side calls only) is left out unless
-- $22 asks for every row; `scoped` keeps it so the page can say how many.
WITH scoped AS MATERIALIZED (
    SELECT f.*, u.display_name, g.name AS group_name, p.name AS project_name,
           f.input_tokens + f.output_tokens AS total_tokens,
           f.cache_read_tokens + f.cache_creation_tokens AS cache_tokens,
           a.status AS judge_status, a.title AS judge_title, a.category, a.summary,
           COALESCE(a.tags, '{}'::text[]) AS tags, COALESCE(a.skills_used, '{}'::text[]) AS skills_used,
           a.outcome, a.completion, a.completion_rationale, a.confidence,
           a.classified_at, a.model AS judge_model, a.cost_microdollars AS judge_cost_microdollars,
           COALESCE(a.input_tokens, 0) + COALESCE(a.output_tokens, 0) AS judge_tokens, a.trigger AS judge_trigger
    FROM conversation_facts f
    LEFT JOIN conversation_analyses a ON a.context_id = f.context_id
    LEFT JOIN users u ON u.id = f.user_id
    LEFT JOIN groups g ON g.id = f.group_id
    LEFT JOIN projects p ON p.id = f.project_id
    WHERE ($1::text[] IS NULL OR f.user_id = ANY($1))
      AND ($2::text IS NULL OR f.user_id = $2)
      AND ($3::text IS NULL OR a.category = $3)
      AND ($4::text IS NULL OR a.outcome = $4)
      AND ($5::text IS NULL OR $5 = ANY(f.skills) OR $5 = ANY(COALESCE(a.skills_used, '{}')))
      AND ($7::timestamptz IS NULL OR f.last_at >= $7)
      AND ($8::timestamptz IS NULL OR f.last_at < $8)
      AND ($14::text IS NULL
           OR ($14 = 'judged' AND a.completion IS NOT NULL)
           OR ($14 = 'unjudged' AND a.completion IS NULL)
           OR ($14 = 'low' AND a.completion < 50)
           OR ($14 = 'high' AND a.completion >= 80))
      AND ($15::text IS NULL OR f.model = $15 OR $15 = ANY(f.models))
      AND ($16::text IS NULL OR f.client_kind = $16)
      AND ($17::text IS NULL OR f.group_id = $17)
      AND ($18::text IS NULL OR f.project_id = $18)
      AND ($19::text IS NULL
           OR ($19 = 'errors' AND f.error_count > 0)
           OR ($19 = 'denied' AND f.gov_deny > 0)
           OR ($19 = 'tools' AND f.tool_calls_intended + f.tool_calls_executed > 0)
           OR ($19 = 'skills' AND cardinality(f.skills) > 0)
           OR ($19 = 'safety' AND f.safety_findings > 0))
      AND ($21::text[] IS NULL OR f.context_id = ANY($21))
      AND ($6::text IS NULL OR u.display_name ILIKE $6 OR f.user_id ILIKE $6 OR a.summary ILIKE $6
           OR a.title ILIKE $6 OR array_to_string(a.tags, ' ') ILIKE $6
           OR array_to_string(f.skills, ' ') ILIKE $6 OR array_to_string(f.models, ' ') ILIKE $6
           OR conversation_title(f.context_id, f.client_session_id) ILIKE $6)
), f AS MATERIALIZED (
    SELECT * FROM scoped WHERE $22::boolean OR turn_count > 0
), totals AS (
    SELECT COUNT(*)::bigint AS conversations, COUNT(DISTINCT user_id)::bigint AS users,
           COALESCE(SUM(turn_count), 0)::bigint AS turns,
           COALESCE(SUM(request_count), 0)::bigint AS requests,
           COALESCE(SUM(tool_calls_intended), 0)::bigint AS tool_calls,
           COALESCE(SUM(tool_calls_executed), 0)::bigint AS tool_calls_executed,
           COALESCE(SUM(tool_calls_failed), 0)::bigint AS tool_calls_failed,
           COALESCE(SUM(input_tokens), 0)::bigint AS input_tokens,
           COALESCE(SUM(output_tokens), 0)::bigint AS output_tokens,
           COALESCE(SUM(cache_tokens), 0)::bigint AS cache_tokens,
           COALESCE(SUM(reasoning_tokens), 0)::bigint AS reasoning_tokens,
           COALESCE(SUM(cost_microdollars), 0)::bigint AS total_cost_microdollars,
           COALESCE(SUM(error_count), 0)::bigint AS errors,
           COALESCE(SUM(rejected_count), 0)::bigint AS rejected,
           COALESCE(SUM(gov_deny), 0)::bigint AS denied,
           COALESCE(SUM(gov_warn), 0)::bigint AS warned,
           COALESCE(SUM(safety_findings), 0)::bigint AS safety_findings,
           COALESCE(SUM(safety_blocked), 0)::bigint AS safety_blocked,
           COALESCE(SUM(artifact_count), 0)::bigint AS artifacts,
           COALESCE(SUM(artifact_files), 0)::bigint AS artifact_files,
           COALESCE(SUM(artifact_cards), 0)::bigint AS artifact_cards,
           COALESCE(SUM(skill_invocations), 0)::bigint AS skill_invocations,
           COALESCE(SUM(active_ms), 0)::bigint AS active_ms,
           COUNT(*) FILTER (WHERE cardinality(skills) > 0)::bigint AS with_skills,
           COUNT(*) FILTER (WHERE outcome = 'achieved')::bigint AS achieved,
           COUNT(*) FILTER (WHERE completion IS NOT NULL)::bigint AS judged,
           AVG(completion)::float8 AS completion_avg,
           percentile_cont(0.95) WITHIN GROUP (ORDER BY p95_latency_ms)::float8 AS p95_latency_ms,
           (SELECT COUNT(*)::bigint FROM conversation_analyses q
             WHERE q.status = 'pending'
               AND ($1::text[] IS NULL OR q.user_id = ANY($1))
               AND ($2::text IS NULL OR q.user_id = $2)) AS pending_judgement,
           (SELECT COUNT(*)::bigint FROM scoped WHERE turn_count = 0) AS without_turns
    FROM f
), buckets AS (
    SELECT generate_series(
        date_trunc($20::text, COALESCE($7::timestamptz,
            GREATEST((SELECT MIN(first_at) FROM f), now() - interval '1 year'), now())),
        date_trunc($20::text, COALESCE($8::timestamptz, now())),
        CASE $20::text WHEN 'hour' THEN interval '1 hour' ELSE interval '1 day' END) AS bucket
), binned AS (
    SELECT date_trunc($20::text, f.first_at) AS bucket,
           COUNT(*)::bigint AS conversations, COALESCE(SUM(f.turn_count), 0)::bigint AS turns,
           COALESCE(SUM(f.total_tokens), 0)::bigint AS tokens,
           COALESCE(SUM(f.cost_microdollars), 0)::bigint AS cost_microdollars,
           COALESCE(SUM(f.error_count), 0)::bigint AS errors,
           COALESCE(SUM(f.tool_calls_intended), 0)::bigint AS tool_calls,
           COALESCE(SUM(f.artifact_count), 0)::bigint AS artifacts,
           COALESCE(SUM(f.gov_deny), 0)::bigint AS denied,
           COUNT(DISTINCT f.user_id)::bigint AS users
    FROM f GROUP BY 1
), series AS (
    SELECT b.bucket, COALESCE(x.conversations, 0)::bigint AS conversations,
           COALESCE(x.turns, 0)::bigint AS turns, COALESCE(x.tokens, 0)::bigint AS tokens,
           COALESCE(x.cost_microdollars, 0)::bigint AS cost_microdollars,
           COALESCE(x.errors, 0)::bigint AS errors, COALESCE(x.tool_calls, 0)::bigint AS tool_calls,
           COALESCE(x.artifacts, 0)::bigint AS artifacts, COALESCE(x.denied, 0)::bigint AS denied,
           COALESCE(x.users, 0)::bigint AS users
    FROM buckets b LEFT JOIN binned x ON x.bucket = b.bucket
    WHERE b.bucket <= now()
    ORDER BY b.bucket
), keyed AS (
    SELECT f.category, f.outcome, f.user_id, f.turn_count, f.total_tokens, f.cost_microdollars,
           f.error_count, f.gov_deny, f.tool_calls_intended, f.artifact_count, f.completion,
           CASE $13::text
               WHEN 'group' THEN COALESCE(f.group_name, 'Unattributed')
               WHEN 'project' THEN COALESCE(f.project_name, 'Unattributed')
               WHEN 'user' THEN COALESCE(f.display_name, f.user_id, 'Unattributed')
               WHEN 'outcome' THEN COALESCE(f.outcome, 'unclear')
               WHEN 'model' THEN COALESCE(f.model, 'unknown')
               WHEN 'client' THEN f.client_kind
               ELSE COALESCE(f.category, 'unjudged') END AS bucket,
           CASE $13::text WHEN 'user' THEN f.user_id ELSE NULL END AS bucket_user
    FROM f WHERE $13::text <> 'skill'
    UNION ALL
    SELECT f.category, f.outcome, f.user_id, f.turn_count, f.total_tokens, f.cost_microdollars,
           f.error_count, f.gov_deny, f.tool_calls_intended, f.artifact_count, f.completion, s.skill, NULL
    FROM f CROSS JOIN LATERAL unnest(f.skills) AS s(skill) WHERE $13::text = 'skill'
), breakdown AS (
    SELECT k.bucket AS label, k.bucket_user AS user_id,
           COUNT(*)::bigint AS conversations, COUNT(DISTINCT k.user_id)::bigint AS users,
           COALESCE(SUM(k.turn_count), 0)::bigint AS turns,
           COALESCE(SUM(k.total_tokens), 0)::bigint AS total_tokens,
           COALESCE(SUM(k.cost_microdollars), 0)::bigint AS total_cost_microdollars,
           COALESCE(SUM(k.error_count), 0)::bigint AS errors,
           COALESCE(SUM(k.gov_deny), 0)::bigint AS denied,
           COALESCE(SUM(k.tool_calls_intended), 0)::bigint AS tool_calls,
           COALESCE(SUM(k.artifact_count), 0)::bigint AS artifacts,
           COUNT(*) FILTER (WHERE k.outcome = 'achieved')::bigint AS achieved,
           COUNT(*) FILTER (WHERE k.completion IS NOT NULL)::bigint AS judged,
           AVG(k.completion)::float8 AS completion_avg,
           mode() WITHIN GROUP (ORDER BY k.category) AS top_category
    FROM keyed k
    GROUP BY k.bucket, k.bucket_user
    ORDER BY total_cost_microdollars DESC, conversations DESC, label
    LIMIT 100
), selected AS MATERIALIZED (
    SELECT f.*, conversation_title(f.context_id, f.client_session_id) AS title,
           ROW_NUMBER() OVER (ORDER BY
             CASE WHEN $9 = 'activity' AND $10::boolean THEN f.last_at END DESC NULLS LAST,
             CASE WHEN $9 = 'activity' AND NOT $10::boolean THEN f.last_at END ASC NULLS LAST,
             CASE WHEN $9 = 'turns' AND $10::boolean THEN f.turn_count END DESC,
             CASE WHEN $9 = 'turns' AND NOT $10::boolean THEN f.turn_count END ASC,
             CASE WHEN $9 = 'tokens' AND $10::boolean THEN f.total_tokens END DESC,
             CASE WHEN $9 = 'tokens' AND NOT $10::boolean THEN f.total_tokens END ASC,
             CASE WHEN $9 = 'cost' AND $10::boolean THEN f.cost_microdollars END DESC,
             CASE WHEN $9 = 'cost' AND NOT $10::boolean THEN f.cost_microdollars END ASC,
             CASE WHEN $9 = 'tools' AND $10::boolean THEN f.tool_calls_intended END DESC,
             CASE WHEN $9 = 'tools' AND NOT $10::boolean THEN f.tool_calls_intended END ASC,
             CASE WHEN $9 = 'errors' AND $10::boolean THEN f.error_count + f.gov_deny END DESC,
             CASE WHEN $9 = 'errors' AND NOT $10::boolean THEN f.error_count + f.gov_deny END ASC,
             CASE WHEN $9 = 'latency' AND $10::boolean THEN f.p95_latency_ms END DESC NULLS LAST,
             CASE WHEN $9 = 'latency' AND NOT $10::boolean THEN f.p95_latency_ms END ASC NULLS LAST,
             CASE WHEN $9 = 'active' AND $10::boolean THEN f.active_ms END DESC,
             CASE WHEN $9 = 'active' AND NOT $10::boolean THEN f.active_ms END ASC,
             CASE WHEN $9 = 'duration' AND $10::boolean THEN f.duration_seconds END DESC,
             CASE WHEN $9 = 'duration' AND NOT $10::boolean THEN f.duration_seconds END ASC,
             CASE WHEN $9 = 'completion' AND $10::boolean THEN f.completion END DESC NULLS LAST,
             CASE WHEN $9 = 'completion' AND NOT $10::boolean THEN f.completion END ASC NULLS LAST,
             f.context_id) AS position
    FROM f
    ORDER BY position
    LIMIT $11 OFFSET $12
), continued AS (
    -- Why: a Claude Code session resumed after compaction opens a new context
    -- whose first prompt is the harness's "continued from a previous
    -- conversation" preamble. Its predecessor is the same person's latest
    -- conversation on the same client that was last active within 30
    -- minutes before it opened.
    SELECT s.context_id, p.context_id AS prev_context_id,
           conversation_title(p.context_id, p.client_session_id) AS prev_title
    FROM selected s
    JOIN LATERAL (
        SELECT q.context_id, q.client_session_id FROM conversation_facts q
        WHERE q.user_id = s.user_id AND q.client_kind = s.client_kind
          AND q.context_id <> s.context_id
          AND q.first_at < s.first_at
          AND q.last_at >= s.first_at - interval '30 minutes'
        ORDER BY (q.client_session_id IS NOT DISTINCT FROM s.client_session_id) DESC,
                 q.last_at DESC, q.context_id
        LIMIT 1) p ON TRUE
    WHERE s.title LIKE 'This session is being continued from a previous%'
), turn_series AS (
    SELECT s.context_id,
           ARRAY(SELECT COALESCE(r.input_tokens, 0) + COALESCE(r.output_tokens, 0)
                 FROM (SELECT cr.input_tokens, cr.output_tokens, cr.created_at, cr.id
                       FROM conversation_requests cr
                       WHERE cr.context_id = s.context_id AND cr.effective_kind = 'turn'
                       ORDER BY cr.created_at DESC, cr.id DESC LIMIT 24) r
                 ORDER BY r.created_at, r.id) AS turn_tokens
    FROM selected s
), skills AS (
    SELECT skill, COUNT(*)::bigint AS conversations FROM (
        SELECT f.context_id, unnest(f.skills) AS skill FROM f
    ) s GROUP BY skill ORDER BY conversations DESC, skill LIMIT 200
), users AS (
    SELECT f.user_id, MAX(f.display_name) AS display_name, COUNT(*)::bigint AS conversations
    FROM f WHERE f.user_id IS NOT NULL GROUP BY f.user_id ORDER BY conversations DESC LIMIT 500
), models AS (
    SELECT m.model, COUNT(*)::bigint AS conversations
    FROM f CROSS JOIN LATERAL unnest(f.models) AS m(model)
    GROUP BY m.model ORDER BY conversations DESC, m.model LIMIT 100
), clients AS (
    SELECT f.client_kind, COUNT(*)::bigint AS conversations
    FROM f GROUP BY f.client_kind ORDER BY conversations DESC
)
SELECT jsonb_build_object(
    'rows', COALESCE((SELECT jsonb_agg(to_jsonb(s) || jsonb_build_object(
                          'turn_tokens', t.turn_tokens,
                          'is_continuation', s.title LIKE 'This session is being continued from a previous%',
                          'prev_context_id', c.prev_context_id, 'prev_title', c.prev_title) ORDER BY s.position)
                      FROM selected s JOIN turn_series t ON t.context_id = s.context_id
                      LEFT JOIN continued c ON c.context_id = s.context_id), '[]'::jsonb),
    'totals', (SELECT to_jsonb(t) FROM totals t),
    'series', COALESCE((SELECT jsonb_agg(to_jsonb(x) ORDER BY x.bucket) FROM series x), '[]'::jsonb),
    'breakdown', COALESCE((SELECT jsonb_agg(to_jsonb(b)) FROM breakdown b), '[]'::jsonb),
    'skills', COALESCE((SELECT jsonb_agg(to_jsonb(k)) FROM skills k), '[]'::jsonb),
    'users', COALESCE((SELECT jsonb_agg(to_jsonb(u)) FROM users u), '[]'::jsonb),
    'models', COALESCE((SELECT jsonb_agg(to_jsonb(m)) FROM models m), '[]'::jsonb),
    'clients', COALESCE((SELECT jsonb_agg(to_jsonb(c)) FROM clients c), '[]'::jsonb)
) AS "payload!: Json<ConversationAnalysisWire>",
    ARRAY(SELECT context_id FROM selected ORDER BY position) AS "context_ids!: Vec<ContextId>"
