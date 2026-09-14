WITH joined AS (
    SELECT
        ar.id, ar.status, ar.latency_ms,
        COALESCE(ar.cost_microdollars, 0)::bigint AS cost_microdollars,
        COALESCE(ar.input_tokens, 0)::bigint  AS input_tokens,
        COALESCE(ar.output_tokens, 0)::bigint AS output_tokens,
        sd.primary_group_id, sd.primary_project_id,
        COALESCE((
            SELECT COUNT(*)::bigint FROM ai_request_tool_calls tc
            WHERE tc.request_id = ar.id
        ), 0) AS tool_call_count,
        COALESCE((
            SELECT COUNT(*)::bigint FROM governance_decisions gd
            WHERE gd.session_id = ar.session_id AND gd.decision = 'deny'
        ), 0) AS deny_count
    FROM ai_requests ar
    LEFT JOIN user_scope_defaults sd ON sd.user_id = ar.user_id
    WHERE ar.created_at >= $1 AND ar.created_at < $2
      AND ($10::TEXT[] IS NULL OR ar.user_id = ANY($10))
      AND ($3::text IS NULL OR ar.user_id = $3)
      AND ($4::text IS NULL OR EXISTS (
          SELECT 1 FROM governance_decisions gd
          WHERE gd.session_id = ar.session_id AND gd.agent_id = $4
      ))
      -- Why: a call the gateway rejected before it resolved a route has
      -- no model and no provider. `unrouted` is the sentinel the
      -- analytics drill-downs send for exactly those rows, so it has
      -- to mean IS NULL rather than a model literally named that.
      AND ($5::text IS NULL
           OR ($5 = 'unrouted' AND ar.model IS NULL)
           OR ar.model = $5)
      AND ($6::text IS NULL
           OR ($6 = 'unrouted' AND ar.provider IS NULL)
           OR ar.provider = $6)
      AND ($7::text IS NULL OR ar.status = $7)
      AND ($8::text IS NULL OR EXISTS (
          SELECT 1 FROM ai_request_tool_calls tc
          WHERE tc.request_id = ar.id AND tc.tool_name = $8
      ))
      AND ($11::text IS NULL OR sd.primary_group_id = $11)
      AND ($12::text IS NULL OR sd.primary_project_id = $12)
      AND ($9::text IS NULL
           OR ar.user_id ILIKE $9
           OR ar.model ILIKE $9
           OR ar.provider ILIKE $9
           OR COALESCE(ar.error_message, '') ILIKE $9
           OR COALESCE(ar.trace_id, '') ILIKE $9)
)
SELECT
    COUNT(*)::bigint AS "total!",
    COUNT(*) FILTER (
        WHERE status NOT IN ('completed', 'pending', 'streaming', 'rejected')
    )::bigint AS "failed!",
    COUNT(*) FILTER (WHERE status = 'rejected')::bigint AS "rejected!",
    COALESCE(SUM(cost_microdollars), 0)::bigint AS "cost_microdollars!",
    COALESCE(SUM(input_tokens), 0)::bigint  AS "input_tokens!",
    COALESCE(SUM(output_tokens), 0)::bigint AS "output_tokens!",
    COALESCE(
        PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY latency_ms), 0
    )::float8 AS "p50_latency_ms!",
    COALESCE(
        PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY latency_ms), 0
    )::float8 AS "p95_latency_ms!",
    COALESCE(SUM(tool_call_count), 0)::bigint AS "tool_calls!",
    COUNT(*) FILTER (WHERE deny_count > 0)::bigint AS "denied!",
    COUNT(*) FILTER (
        WHERE primary_group_id IS NULL AND primary_project_id IS NULL
    )::bigint AS "unattributed!"
FROM joined
