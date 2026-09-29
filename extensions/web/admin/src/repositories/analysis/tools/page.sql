-- One statement for the Tools and Artifacts pages over the tool_activity
-- view: the filtered set once, then its totals, its time series padded to
-- every bucket of the window, one breakdown dimension, the facet lists and
-- one page of rows. The Artifacts page is the same statement with
-- $14 (artifacts only) true. Filtering, paging and sorting are bound
-- parameters chosen by CASE arms, never interpolated text. A host-native
-- tool (Bash, Read, …) has no MCP server, so the server facet and the
-- server breakdown leave builtins out unless $7 asks for them; totals count
-- every row.
WITH f AS MATERIALIZED (
    SELECT t.*,
           COALESCE(t.mcp_execution_id, t.ai_tool_call_id, t.intent_id) AS row_key,
           COALESCE(u.display_name, u.full_name, u.name, u.email) AS display_name,
           g.decision, g.id AS decision_id,
           (SELECT s.skill FROM analysis_skill_events s
             WHERE s.session_id = t.execution_trace_id AND s.skill IS NOT NULL
               AND s.invoked_at <= COALESCE(t.occurred_at, clock_timestamp())
             ORDER BY s.invoked_at DESC LIMIT 1) AS skill,
           -- Why: `execution_status` is NULL on an intent with no execution, and
           -- `FALSE OR NULL` is NULL — a null the row type's `bool` cannot decode.
           (COALESCE(t.is_error, FALSE) OR COALESCE(t.execution_status, '') IN ('failed', 'timeout')) AS failed
    FROM tool_activity t
    LEFT JOIN users u ON u.id = t.user_id
    LEFT JOIN LATERAL (
        SELECT d.id, d.decision FROM governance_decisions d
        WHERE d.tool_use_id IS NOT NULL AND d.tool_use_id = t.ai_tool_call_id
        ORDER BY d.created_at DESC LIMIT 1
    ) g ON TRUE
    WHERE ($1::timestamptz IS NULL OR t.occurred_at >= $1)
      AND ($2::timestamptz IS NULL OR t.occurred_at < $2)
      AND ($3::text[] IS NULL OR t.user_id = ANY($3))
      AND ($4::text IS NULL OR t.user_id = $4)
      AND ($5::text IS NULL OR t.tool_name = $5)
      AND ($6::text IS NULL OR t.server_name = $6)
      AND ($7::boolean IS NULL OR t.is_builtin = $7)
      AND ($8::text IS NULL
           OR ($8 = 'failed' AND (COALESCE(t.is_error, FALSE) OR COALESCE(t.execution_status, '') IN ('failed', 'timeout')))
           OR ($8 = 'executed' AND t.state = 'executed' AND NOT (COALESCE(t.is_error, FALSE) OR COALESCE(t.execution_status, '') IN ('failed', 'timeout')))
           OR ($8 = 'intended' AND t.state = 'intended')
           OR ($8 = 'unattested' AND t.state = 'unattested'))
      AND ($9::text IS NULL OR g.decision = $9)
      AND ($10::text IS NULL OR t.context_id = $10 OR t.execution_context_id = $10)
      AND ($11::text IS NULL OR t.session_id = $11 OR t.execution_trace_id = $11)
      AND ($13::text IS NULL OR t.artifact_kind = $13)
      AND ($23::text IS NULL OR t.client_kind = $23)
      AND (NOT $14::boolean OR t.artifact_kind IS NOT NULL)
      AND ($15::text IS NULL OR t.tool_name ILIKE $15 OR t.server_name ILIKE $15
           OR t.input_summary ILIKE $15 OR t.artifact_title ILIKE $15)
      AND ($16::text[] IS NULL OR COALESCE(t.mcp_execution_id, t.ai_tool_call_id, t.intent_id) = ANY($16))
), s AS MATERIALIZED (
    SELECT * FROM f WHERE ($12::text IS NULL OR f.skill = $12)
), totals AS (
    SELECT COUNT(*)::bigint AS calls,
           COUNT(*) FILTER (WHERE state = 'executed')::bigint AS executed,
           COUNT(*) FILTER (WHERE failed)::bigint AS failed,
           COUNT(*) FILTER (WHERE decision = 'deny')::bigint AS denied,
           COUNT(*) FILTER (WHERE decision = 'warn')::bigint AS warned,
           COUNT(*) FILTER (WHERE state = 'intended')::bigint AS intended,
           COUNT(*) FILTER (WHERE state = 'unattested')::bigint AS unattested,
           COUNT(*) FILTER (WHERE is_builtin)::bigint AS builtin,
           COUNT(DISTINCT tool_name)::bigint AS tools,
           COUNT(DISTINCT server_name)::bigint AS servers,
           COUNT(DISTINCT user_id)::bigint AS users,
           COUNT(DISTINCT COALESCE(context_id, execution_context_id))::bigint AS conversations,
           percentile_cont(0.95) WITHIN GROUP (ORDER BY execution_time_ms)::float8 AS p95_duration_ms,
           COUNT(*) FILTER (WHERE artifact_kind IS NOT NULL)::bigint AS artifacts,
           COUNT(*) FILTER (WHERE artifact_kind = 'file')::bigint AS files,
           COUNT(*) FILTER (WHERE artifact_kind = 'card')::bigint AS cards,
           COUNT(*) FILTER (WHERE artifact_kind = 'ui')::bigint AS ui,
           COUNT(*) FILTER (WHERE artifact_kind = 'body')::bigint AS bodies,
           COUNT(*) FILTER (WHERE artifact_kind IS NOT NULL AND is_error)::bigint AS artifact_errors,
           COALESCE(SUM(payload_bytes) FILTER (WHERE artifact_kind IS NOT NULL), 0)::bigint AS bytes,
           COALESCE(SUM(secret_redactions), 0)::bigint AS redactions
    FROM s
), buckets AS (
    SELECT generate_series(date_trunc($22::text, COALESCE($1::timestamptz, (SELECT MIN(occurred_at) FROM s), clock_timestamp())),
                           date_trunc($22::text, COALESCE($2::timestamptz, clock_timestamp())),
                           CASE WHEN $22::text = 'hour' THEN interval '1 hour' ELSE interval '1 day' END) AS bucket
), series AS (
    SELECT b.bucket,
           COUNT(s.row_key)::bigint AS calls,
           COUNT(s.row_key) FILTER (WHERE s.state = 'executed')::bigint AS executed,
           COUNT(s.row_key) FILTER (WHERE s.failed)::bigint AS failed,
           COUNT(s.row_key) FILTER (WHERE s.decision = 'deny')::bigint AS denied,
           COUNT(s.row_key) FILTER (WHERE s.artifact_kind = 'file')::bigint AS files,
           COUNT(s.row_key) FILTER (WHERE s.artifact_kind = 'card')::bigint AS cards,
           COUNT(s.row_key) FILTER (WHERE s.artifact_kind = 'ui')::bigint AS ui,
           COUNT(s.row_key) FILTER (WHERE s.artifact_kind = 'body')::bigint AS bodies,
           COUNT(s.row_key) FILTER (WHERE s.artifact_kind IS NOT NULL)::bigint AS artifacts
    FROM buckets b
    LEFT JOIN s ON date_trunc($22::text, s.occurred_at) = b.bucket
    GROUP BY b.bucket ORDER BY b.bucket
), keyed AS (
    SELECT s.*,
           CASE $21::text
               WHEN 'server' THEN COALESCE(s.server_name, 'unknown')
               WHEN 'user' THEN COALESCE(s.display_name, s.user_id, 'Unattributed')
               WHEN 'client' THEN COALESCE(s.client_kind, 'unknown')
               WHEN 'skill' THEN COALESCE(s.skill, 'no skill')
               WHEN 'kind' THEN COALESCE(s.artifact_kind, 'tool call')
               ELSE COALESCE(s.tool_name, 'unknown') END AS bucket,
           CASE $21::text WHEN 'user' THEN s.user_id WHEN 'skill' THEN s.skill
                          WHEN 'kind' THEN s.artifact_kind WHEN 'server' THEN s.server_name
                          WHEN 'client' THEN s.client_kind ELSE s.tool_name END AS bucket_value
    FROM s
), breakdown AS (
    SELECT k.bucket AS label, k.bucket_value AS value,
           COUNT(*)::bigint AS calls,
           COUNT(*) FILTER (WHERE k.state = 'executed')::bigint AS executed,
           COUNT(*) FILTER (WHERE k.failed)::bigint AS failed,
           COUNT(*) FILTER (WHERE k.decision = 'deny')::bigint AS denied,
           COUNT(DISTINCT k.user_id)::bigint AS users,
           COUNT(*) FILTER (WHERE k.artifact_kind IS NOT NULL)::bigint AS artifacts,
           percentile_cont(0.95) WITHIN GROUP (ORDER BY k.execution_time_ms)::float8 AS p95_duration_ms,
           COALESCE(SUM(k.payload_bytes) FILTER (WHERE k.artifact_kind IS NOT NULL), 0)::bigint AS bytes
    FROM keyed k
    WHERE NOT ($21::text = 'server' AND k.is_builtin AND $7::boolean IS DISTINCT FROM TRUE)
    GROUP BY k.bucket, k.bucket_value
    ORDER BY calls DESC, label
    LIMIT 100
), selected AS MATERIALIZED (
    SELECT s.*,
           ROW_NUMBER() OVER (ORDER BY
             CASE WHEN $17 = 'time' AND $18::boolean THEN s.occurred_at END DESC NULLS LAST,
             CASE WHEN $17 = 'time' AND NOT $18::boolean THEN s.occurred_at END ASC NULLS LAST,
             CASE WHEN $17 = 'duration' AND $18::boolean THEN s.execution_time_ms END DESC NULLS LAST,
             CASE WHEN $17 = 'duration' AND NOT $18::boolean THEN s.execution_time_ms END ASC NULLS LAST,
             CASE WHEN $17 = 'tool' AND $18::boolean THEN s.tool_name END DESC NULLS LAST,
             CASE WHEN $17 = 'tool' AND NOT $18::boolean THEN s.tool_name END ASC NULLS LAST,
             CASE WHEN $17 = 'size' AND $18::boolean THEN s.payload_bytes END DESC NULLS LAST,
             CASE WHEN $17 = 'size' AND NOT $18::boolean THEN s.payload_bytes END ASC NULLS LAST,
             s.row_key) AS position
    FROM s ORDER BY position LIMIT $19 OFFSET $20
), tools AS (
    SELECT tool_name AS value, COUNT(*)::bigint AS calls FROM s WHERE tool_name IS NOT NULL
    GROUP BY tool_name ORDER BY calls DESC, value LIMIT 200
), servers AS (
    SELECT server_name AS value, COUNT(*)::bigint AS calls FROM s
    WHERE server_name IS NOT NULL AND ($7::boolean IS TRUE OR NOT is_builtin)
    GROUP BY server_name ORDER BY calls DESC, value LIMIT 100
), users AS (
    SELECT user_id AS value, MAX(display_name) AS label, COUNT(*)::bigint AS calls FROM s
    WHERE user_id IS NOT NULL GROUP BY user_id ORDER BY calls DESC LIMIT 500
), clients AS (
    SELECT client_kind AS value, COUNT(*)::bigint AS calls FROM s WHERE client_kind IS NOT NULL
    GROUP BY client_kind ORDER BY calls DESC
), skills AS (
    SELECT skill AS value, COUNT(*)::bigint AS calls FROM f WHERE skill IS NOT NULL
    GROUP BY skill ORDER BY calls DESC, value LIMIT 200
), kinds AS (
    SELECT artifact_kind AS value, COUNT(*)::bigint AS calls FROM s WHERE artifact_kind IS NOT NULL
    GROUP BY artifact_kind ORDER BY calls DESC
), decisions AS (
    SELECT decision AS value, COUNT(*)::bigint AS calls FROM s WHERE decision IS NOT NULL
    GROUP BY decision ORDER BY calls DESC
)
SELECT jsonb_build_object(
    'rows', COALESCE((SELECT jsonb_agg(to_jsonb(x) ORDER BY x.position) FROM selected x), '[]'::jsonb),
    'totals', (SELECT to_jsonb(t) FROM totals t),
    'series', COALESCE((SELECT jsonb_agg(to_jsonb(x) ORDER BY x.bucket) FROM series x), '[]'::jsonb),
    'breakdown', COALESCE((SELECT jsonb_agg(to_jsonb(b)) FROM breakdown b), '[]'::jsonb),
    'tools', COALESCE((SELECT jsonb_agg(to_jsonb(x)) FROM tools x), '[]'::jsonb),
    'servers', COALESCE((SELECT jsonb_agg(to_jsonb(x)) FROM servers x), '[]'::jsonb),
    'users', COALESCE((SELECT jsonb_agg(to_jsonb(x)) FROM users x), '[]'::jsonb),
    'clients', COALESCE((SELECT jsonb_agg(to_jsonb(x)) FROM clients x), '[]'::jsonb),
    'skills', COALESCE((SELECT jsonb_agg(to_jsonb(x)) FROM skills x), '[]'::jsonb),
    'kinds', COALESCE((SELECT jsonb_agg(to_jsonb(x)) FROM kinds x), '[]'::jsonb),
    'decisions', COALESCE((SELECT jsonb_agg(to_jsonb(x)) FROM decisions x), '[]'::jsonb)
) AS "payload!: Json<ToolActivityResult>"
