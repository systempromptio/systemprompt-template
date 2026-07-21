//! Trace list query: one aggregated summary row per session in the window.

use sqlx::PgPool;
use systemprompt::identifiers::{AgentId, SessionId, TraceId, UserId};

use super::{TraceFilter, TraceSort, TraceSummary};
use crate::util::time_range::TimeRange;

#[derive(Debug)]
struct TraceRow {
    session_id: SessionId,
    trace_id: Option<TraceId>,
    started_at: chrono::DateTime<chrono::Utc>,
    ended_at: chrono::DateTime<chrono::Utc>,
    duration_ms: i64,
    user_id: Option<UserId>,
    agent_id: Option<AgentId>,
    agent_scope: Option<String>,
    model: Option<String>,
    provider: Option<String>,
    span_count: i64,
    request_count: i64,
    tool_call_count: i64,
    governance_count: i64,
    deny_count: i64,
    total_tokens: i64,
    input_tokens: i64,
    output_tokens: i64,
    total_cost_microdollars: i64,
    total_latency_ms: i64,
    cache_hit_any: bool,
    top_tool: Option<String>,
    has_error: bool,
    has_deny: bool,
    total_count: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct TracePage {
    pub sort: TraceSort,
    pub limit: i64,
    pub offset: i64,
}

/// The sort is a closed `TraceSort` (five columns × two directions).
///
/// Each `(column, dir)` pair is bound as text and selected by a per-key `CASE`
/// in the `ORDER BY`, so the whole statement stays a single compile-time
/// `query_as!` rather than an interpolated string.
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
        TraceRow,
        r#"WITH trace_to_session AS (
            SELECT DISTINCT trace_id, session_id
            FROM ai_requests
            WHERE created_at >= $1 AND created_at < $2
              AND trace_id IS NOT NULL AND session_id IS NOT NULL
        ),
        all_sessions AS (
            SELECT
                COALESCE(t.session_id, g.session_id) AS session_id,
                g.user_id, g.agent_id, g.agent_scope,
                g.created_at, g.decision, 'gov'::text AS source
            FROM governance_decisions g
            LEFT JOIN trace_to_session t ON t.trace_id = g.session_id
            WHERE g.created_at >= $1 AND g.created_at < $2
              AND g.session_id IS NOT NULL
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
        ai_meta AS (
            SELECT
                session_id,
                (ARRAY_AGG(trace_id ORDER BY created_at DESC))[1]   AS trace_id,
                (ARRAY_AGG(model    ORDER BY created_at DESC))[1]   AS model,
                (ARRAY_AGG(provider ORDER BY created_at DESC))[1]   AS provider,
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
        tool_meta AS (
            SELECT
                session_id,
                COUNT(*)::bigint                                    AS tool_call_count,
                MODE() WITHIN GROUP (ORDER BY tool_name)            AS top_tool
            FROM plugin_usage_events
            WHERE created_at >= $1 AND created_at < $2
              AND session_id IS NOT NULL
              AND tool_name IS NOT NULL
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
                GREATEST(
                    (EXTRACT(EPOCH FROM (p.ended_at - p.started_at)) * 1000)::bigint,
                    COALESCE(a.total_latency_ms, 0)
                )                                                   AS duration_ms,
                p.span_count,
                COALESCE(a.request_count, 0)        AS request_count,
                COALESCE(t.tool_call_count, 0)      AS tool_call_count,
                p.governance_count,
                p.deny_count,
                (p.deny_count > 0)                  AS has_deny,
                a.trace_id,
                a.model,
                a.provider,
                COALESCE(a.total_tokens, 0)         AS total_tokens,
                COALESCE(a.input_tokens, 0)         AS input_tokens,
                COALESCE(a.output_tokens, 0)        AS output_tokens,
                COALESCE(a.total_cost_microdollars, 0) AS total_cost_microdollars,
                COALESCE(a.total_latency_ms, 0)     AS total_latency_ms,
                COALESCE(a.cache_hit_any, false)    AS cache_hit_any,
                t.top_tool,
                COALESCE(a.has_error, false)        AS has_error
            FROM per_session p
            LEFT JOIN ai_meta   a ON a.session_id = p.session_id
            LEFT JOIN tool_meta t ON t.session_id = p.session_id
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
        counted AS (
            SELECT
                f.*,
                COUNT(*) OVER ()::bigint AS total_count
            FROM filtered f
        )
        SELECT
            session_id              AS "session_id!: SessionId",
            trace_id                AS "trace_id?: TraceId",
            started_at              AS "started_at!",
            ended_at                AS "ended_at!",
            duration_ms             AS "duration_ms!",
            user_id                 AS "user_id?: UserId",
            agent_id                AS "agent_id?: AgentId",
            agent_scope             AS "agent_scope?",
            model                   AS "model?",
            provider                AS "provider?",
            span_count              AS "span_count!",
            request_count           AS "request_count!",
            tool_call_count         AS "tool_call_count!",
            governance_count        AS "governance_count!",
            deny_count              AS "deny_count!",
            total_tokens            AS "total_tokens!",
            input_tokens            AS "input_tokens!",
            output_tokens           AS "output_tokens!",
            total_cost_microdollars AS "total_cost_microdollars!",
            total_latency_ms        AS "total_latency_ms!",
            cache_hit_any           AS "cache_hit_any!",
            top_tool                AS "top_tool?",
            has_error               AS "has_error!",
            has_deny                AS "has_deny!",
            total_count             AS "total_count!"
        FROM counted
        ORDER BY
            (CASE WHEN $12 = 'started_at' AND $13 = 'asc'  THEN started_at END) ASC  NULLS LAST,
            (CASE WHEN $12 = 'started_at' AND $13 = 'desc' THEN started_at END) DESC NULLS LAST,
            (CASE WHEN $12 = 'duration'   AND $13 = 'asc'  THEN duration_ms END) ASC  NULLS LAST,
            (CASE WHEN $12 = 'duration'   AND $13 = 'desc' THEN duration_ms END) DESC NULLS LAST,
            (CASE WHEN $12 = 'span_count' AND $13 = 'asc'  THEN span_count  END) ASC  NULLS LAST,
            (CASE WHEN $12 = 'span_count' AND $13 = 'desc' THEN span_count  END) DESC NULLS LAST,
            (CASE WHEN $12 = 'cost'       AND $13 = 'asc'  THEN total_cost_microdollars END) ASC  NULLS LAST,
            (CASE WHEN $12 = 'cost'       AND $13 = 'desc' THEN total_cost_microdollars END) DESC NULLS LAST,
            (CASE WHEN $12 = 'tokens'     AND $13 = 'asc'  THEN total_tokens END) ASC  NULLS LAST,
            (CASE WHEN $12 = 'tokens'     AND $13 = 'desc' THEN total_tokens END) DESC NULLS LAST
        LIMIT $10 OFFSET $11"#,
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

impl From<TraceRow> for TraceSummary {
    fn from(r: TraceRow) -> Self {
        Self {
            session_id: r.session_id,
            trace_id: r.trace_id,
            started_at: r.started_at,
            ended_at: r.ended_at,
            duration_ms: r.duration_ms,
            user_id: r.user_id,
            agent_id: r.agent_id,
            agent_scope: r.agent_scope,
            model: r.model,
            provider: r.provider,
            span_count: r.span_count,
            request_count: r.request_count,
            tool_call_count: r.tool_call_count,
            governance_count: r.governance_count,
            deny_count: r.deny_count,
            total_tokens: r.total_tokens,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            total_cost_microdollars: r.total_cost_microdollars,
            total_latency_ms: r.total_latency_ms,
            cache_hit_any: r.cache_hit_any,
            top_tool: r.top_tool,
            has_error: r.has_error,
            has_deny: r.has_deny,
        }
    }
}
