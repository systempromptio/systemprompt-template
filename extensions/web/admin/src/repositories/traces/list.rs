//! Trace list query: one aggregated summary row per session in the window.

use sqlx::PgPool;
// Why: the `query_as!` column overrides below name these types, so they must be
// in scope here even though the row struct itself lives in `list_row`.
use systemprompt::identifiers::{AgentId, SessionId, TraceId, UserId};

use super::list_row::TraceListRow;
use super::{TraceFilter, TraceSort, TraceSummary};
use crate::util::time_range::TimeRange;

#[derive(Debug, Clone, Copy)]
pub struct TracePage {
    pub sort: TraceSort,
    pub limit: i64,
    pub offset: i64,
}

// Why: The sort is a closed `TraceSort` (five columns × two directions).
//
// Each `(column, dir)` pair is bound as text and selected by a per-key `CASE`
// in the `ORDER BY`, so the whole statement stays a single compile-time
// `query_as!` rather than an interpolated string.
#[expect(
    clippy::too_many_lines,
    reason = "body is one irreducible compile-time-checked query_as! SQL literal; see comment above"
)]
pub async fn list_traces(
    pool: &PgPool,
    filter: TraceFilter<'_>,
    range: TimeRange,
    page: TracePage,
) -> Result<(Vec<TraceSummary>, i64), sqlx::Error> {
    let TracePage {
        sort,
        limit,
        offset,
    } = page;
    let sort_col = sort.column.sql_key();
    let sort_dir = sort.dir.sql_key();

    let rows = sqlx::query_as!(
        TraceListRow,
        r#"WITH trace_to_session AS (
            SELECT DISTINCT trace_id, session_id
            FROM ai_requests
            WHERE created_at >= $1 AND created_at < $2
              AND trace_id IS NOT NULL AND session_id IS NOT NULL
        ),
        all_sessions AS (
            SELECT
                COALESCE(t.session_id, NULLIF(g.session_id, ''), g.trace_id) AS session_id,
                g.user_id, g.agent_id, g.agent_scope,
                g.created_at, g.decision, 'gov'::text AS source
            FROM governance_decisions g
            LEFT JOIN trace_to_session t ON t.trace_id = g.trace_id
            WHERE g.created_at >= $1 AND g.created_at < $2
              AND (NULLIF(g.session_id, '') IS NOT NULL OR g.trace_id IS NOT NULL)
            UNION ALL
            SELECT session_id, user_id, NULL::text AS agent_id, NULL::text AS agent_scope,
                   created_at, NULL::text AS decision, 'ai'::text AS source
            FROM ai_requests
            WHERE created_at >= $1 AND created_at < $2
              AND session_id IS NOT NULL
            UNION ALL
            SELECT session_id, user_id, NULL::text AS agent_id, NULL::text AS agent_scope,
                   created_at, NULL::text AS decision, 'evt'::text AS source
            FROM plugin_usage_events
            WHERE created_at >= $1 AND created_at < $2
              AND session_id IS NOT NULL
        ),
        per_session AS (
            SELECT
                session_id,
                MAX(user_id)               AS user_id,
                MAX(agent_id)              AS agent_id,
                MAX(agent_scope)           AS agent_scope,
                MIN(created_at)            AS started_at,
                MAX(created_at)            AS ended_at,
                COUNT(*)::bigint           AS span_count,
                COUNT(*) FILTER (WHERE source = 'gov')::bigint        AS governance_count,
                COUNT(*) FILTER (WHERE decision = 'deny')::bigint     AS deny_count
            FROM all_sessions
            GROUP BY session_id
        ),
        -- Stage 1 aggregates: only what filtering and sorting need. The
        -- expensive per-session picks (ARRAY_AGG / MODE / users lateral) run
        -- in stage 2 over the selected page alone.
        ai_sums AS (
            SELECT
                session_id,
                COUNT(*)::bigint                                    AS request_count,
                COALESCE(SUM(tokens_used), 0)::bigint               AS total_tokens,
                COALESCE(SUM(input_tokens), 0)::bigint              AS input_tokens,
                COALESCE(SUM(output_tokens), 0)::bigint             AS output_tokens,
                COALESCE(SUM(cost_microdollars), 0)::bigint         AS total_cost_microdollars,
                COALESCE(SUM(latency_ms), 0)::bigint                AS total_latency_ms,
                BOOL_OR(cache_hit)                                  AS cache_hit_any,
                BOOL_OR(status NOT IN ('ok', 'success', 'completed', 'pending'))
                                                                    AS has_error
            FROM ai_requests
            WHERE created_at >= $1 AND created_at < $2
              AND session_id IS NOT NULL
            GROUP BY session_id
        ),
        joined AS (
            SELECT
                p.session_id,
                p.user_id,
                p.agent_id,
                p.agent_scope,
                p.started_at,
                p.ended_at,
                -- Two distinct clocks, never collapsed: `active_ms` is the work
                -- actually done (summed request latency), `window_ms` the span
                -- between first and last event on a session id that a client is
                -- free to reuse for hours.
                COALESCE(a.total_latency_ms, 0)     AS active_ms,
                (EXTRACT(EPOCH FROM (p.ended_at - p.started_at)) * 1000)::bigint
                                                    AS window_ms,
                p.span_count,
                COALESCE(a.request_count, 0)        AS request_count,
                p.governance_count,
                p.deny_count,
                (p.deny_count > 0)                  AS has_deny,
                COALESCE(a.total_tokens, 0)         AS total_tokens,
                COALESCE(a.input_tokens, 0)         AS input_tokens,
                COALESCE(a.output_tokens, 0)        AS output_tokens,
                COALESCE(a.total_cost_microdollars, 0) AS total_cost_microdollars,
                COALESCE(a.total_latency_ms, 0)     AS total_latency_ms,
                COALESCE(a.cache_hit_any, false)    AS cache_hit_any,
                COALESCE(a.has_error, false)        AS has_error
            FROM per_session p
            LEFT JOIN ai_sums a ON a.session_id = p.session_id
        ),
        filtered AS (
            SELECT j.* FROM joined j
            WHERE ($3::text  IS NULL OR j.user_id     = $3)
              AND ($4::text  IS NULL OR j.agent_id    = $4)
              AND ($5::text  IS NULL OR j.agent_scope = $5)
              AND ($6::text  IS NULL OR EXISTS (
                    SELECT 1 FROM governance_decisions g
                    WHERE g.session_id = j.session_id
                      AND g.created_at >= $1 AND g.created_at < $2
                      AND g.policy = $6))
              AND ($7::text  IS NULL OR EXISTS (
                    SELECT 1 FROM governance_decisions g
                    WHERE g.session_id = j.session_id
                      AND g.created_at >= $1 AND g.created_at < $2
                      AND g.decision = $7))
              AND (NOT $8 OR j.has_error = true)
              AND (NOT $9 OR j.has_deny  = true)
        ),
        -- The page is chosen here, so everything below runs for at most
        -- `limit` sessions. The session_id tiebreaker makes ties (and thus
        -- pages) deterministic; the outer ORDER BY repeats it because a join
        -- does not preserve order.
        page AS (
            SELECT
                f.*,
                COUNT(*) OVER ()::bigint AS total_count
            FROM filtered f
            ORDER BY
                (CASE WHEN $12 = 'started_at' AND $13 = 'asc'  THEN f.started_at END) ASC  NULLS LAST,
                (CASE WHEN $12 = 'started_at' AND $13 = 'desc' THEN f.started_at END) DESC NULLS LAST,
                (CASE WHEN $12 = 'duration'   AND $13 = 'asc'  THEN f.active_ms END) ASC  NULLS LAST,
                (CASE WHEN $12 = 'duration'   AND $13 = 'desc' THEN f.active_ms END) DESC NULLS LAST,
                (CASE WHEN $12 = 'span_count' AND $13 = 'asc'  THEN f.span_count  END) ASC  NULLS LAST,
                (CASE WHEN $12 = 'span_count' AND $13 = 'desc' THEN f.span_count  END) DESC NULLS LAST,
                (CASE WHEN $12 = 'cost'       AND $13 = 'asc'  THEN f.total_cost_microdollars END) ASC  NULLS LAST,
                (CASE WHEN $12 = 'cost'       AND $13 = 'desc' THEN f.total_cost_microdollars END) DESC NULLS LAST,
                (CASE WHEN $12 = 'tokens'     AND $13 = 'asc'  THEN f.total_tokens END) ASC  NULLS LAST,
                (CASE WHEN $12 = 'tokens'     AND $13 = 'desc' THEN f.total_tokens END) DESC NULLS LAST,
                f.session_id ASC
            LIMIT $10 OFFSET $11
        ),
        ai_picks AS (
            SELECT
                session_id,
                (ARRAY_AGG(trace_id ORDER BY created_at DESC))[1]   AS trace_id,
                (ARRAY_AGG(model    ORDER BY created_at DESC))[1]   AS model,
                (ARRAY_AGG(provider ORDER BY created_at DESC))[1]   AS provider
            FROM ai_requests
            WHERE created_at >= $1 AND created_at < $2
              AND session_id IN (SELECT session_id FROM page)
            GROUP BY session_id
        ),
        tool_meta AS (
            SELECT
                session_id,
                COUNT(*)::bigint                                    AS tool_call_count,
                MODE() WITHIN GROUP (ORDER BY tool_name)            AS top_tool
            FROM plugin_usage_events
            WHERE created_at >= $1 AND created_at < $2
              AND session_id IN (SELECT session_id FROM page)
              AND tool_name IS NOT NULL
            GROUP BY session_id
        )
        SELECT
            pg.session_id           AS "session_id!: SessionId",
            k.trace_id              AS "trace_id?: TraceId",
            pg.started_at           AS "started_at!",
            pg.ended_at             AS "ended_at!",
            pg.active_ms            AS "active_ms!",
            pg.window_ms            AS "window_ms!",
            pg.user_id              AS "user_id?: UserId",
            u.label                 AS "user_label?",
            pg.agent_id             AS "agent_id?: AgentId",
            pg.agent_scope          AS "agent_scope?",
            k.model                 AS "model?",
            k.provider              AS "provider?",
            pg.span_count           AS "span_count!",
            pg.request_count        AS "request_count!",
            COALESCE(t.tool_call_count, 0) AS "tool_call_count!",
            pg.governance_count     AS "governance_count!",
            pg.deny_count           AS "deny_count!",
            pg.total_tokens         AS "total_tokens!",
            pg.input_tokens         AS "input_tokens!",
            pg.output_tokens        AS "output_tokens!",
            pg.total_cost_microdollars AS "total_cost_microdollars!",
            pg.total_latency_ms     AS "total_latency_ms!",
            pg.cache_hit_any        AS "cache_hit_any!",
            t.top_tool              AS "top_tool?",
            pg.has_error            AS "has_error!",
            pg.has_deny             AS "has_deny!",
            pg.total_count          AS "total_count!"
        FROM page pg
        LEFT JOIN ai_picks k ON k.session_id = pg.session_id
        LEFT JOIN tool_meta t ON t.session_id = pg.session_id
        LEFT JOIN LATERAL (
            SELECT COALESCE(x.display_name, x.full_name, x.name, x.email) AS label
            FROM users x WHERE x.id = pg.user_id
        ) u ON true
        ORDER BY
            (CASE WHEN $12 = 'started_at' AND $13 = 'asc'  THEN pg.started_at END) ASC  NULLS LAST,
            (CASE WHEN $12 = 'started_at' AND $13 = 'desc' THEN pg.started_at END) DESC NULLS LAST,
            (CASE WHEN $12 = 'duration'   AND $13 = 'asc'  THEN pg.active_ms END) ASC  NULLS LAST,
            (CASE WHEN $12 = 'duration'   AND $13 = 'desc' THEN pg.active_ms END) DESC NULLS LAST,
            (CASE WHEN $12 = 'span_count' AND $13 = 'asc'  THEN pg.span_count  END) ASC  NULLS LAST,
            (CASE WHEN $12 = 'span_count' AND $13 = 'desc' THEN pg.span_count  END) DESC NULLS LAST,
            (CASE WHEN $12 = 'cost'       AND $13 = 'asc'  THEN pg.total_cost_microdollars END) ASC  NULLS LAST,
            (CASE WHEN $12 = 'cost'       AND $13 = 'desc' THEN pg.total_cost_microdollars END) DESC NULLS LAST,
            (CASE WHEN $12 = 'tokens'     AND $13 = 'asc'  THEN pg.total_tokens END) ASC  NULLS LAST,
            (CASE WHEN $12 = 'tokens'     AND $13 = 'desc' THEN pg.total_tokens END) DESC NULLS LAST,
            pg.session_id ASC"#,
        range.from,
        range.to,
        filter.user_id,
        filter.agent_id,
        filter.agent_scope,
        filter.policy,
        filter.decision,
        filter.error_only,
        filter.deny_only,
        limit,
        offset,
        sort_col,
        sort_dir,
    )
    .fetch_all(pool)
    .await?;

    let total = rows.first().map_or(0, |r| r.total_count);
    let summaries = rows.into_iter().map(TraceSummary::from).collect();
    Ok((summaries, total))
}
