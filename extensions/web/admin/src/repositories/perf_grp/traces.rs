//! Trace explorer queries.
//!
//! A "trace" here is keyed on `session_id` (which all four governance / gateway
//! / events tables share) and surfaces the `trace_id` from `ai_requests` when
//! one exists. `fetch_trace_list` returns one summary row per session in the
//! window; `fetch_trace_spans` returns the union of per-table rows for a
//! single session, normalised into a `Span` shape and ordered by start time.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

use crate::repositories::governance_grp::time_range::TimeRange;

/// One row per session in the window — feeds the list page.
#[derive(Debug, Clone, Serialize)]
pub struct TraceSummary {
    pub session_id: String,
    pub trace_id: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub duration_ms: i64,
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub agent_scope: Option<String>,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub span_count: i64,
    pub request_count: i64,
    pub tool_call_count: i64,
    pub governance_count: i64,
    pub deny_count: i64,
    pub total_tokens: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_cost_microdollars: i64,
    pub total_latency_ms: i64,
    pub cache_hit_any: bool,
    pub top_tool: Option<String>,
    pub has_error: bool,
    pub has_deny: bool,
}

/// Raw row returned by the dynamic `fetch_trace_list` query.
#[derive(Debug, sqlx::FromRow)]
struct TraceRow {
    session_id: String,
    trace_id: Option<String>,
    started_at: DateTime<Utc>,
    ended_at: DateTime<Utc>,
    duration_ms: i64,
    user_id: Option<String>,
    agent_id: Option<String>,
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

#[derive(Debug, Clone, Copy, Default)]
pub struct TraceFilter<'a> {
    pub user_id: Option<&'a str>,
    pub agent_id: Option<&'a str>,
    pub agent_scope: Option<&'a str>,
    pub policy: Option<&'a str>,
    pub decision: Option<&'a str>,
    pub error_only: bool,
    pub deny_only: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum TraceSortColumn {
    StartedAt,
    Duration,
    SpanCount,
    Cost,
    Tokens,
}

#[derive(Debug, Clone, Copy)]
pub enum TraceSortDir {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy)]
pub struct TraceSort {
    pub column: TraceSortColumn,
    pub dir: TraceSortDir,
}

impl Default for TraceSort {
    fn default() -> Self {
        Self {
            column: TraceSortColumn::StartedAt,
            dir: TraceSortDir::Desc,
        }
    }
}

const fn order_by_clause(sort: TraceSort) -> &'static str {
    match (sort.column, sort.dir) {
        (TraceSortColumn::StartedAt, TraceSortDir::Asc) => "started_at ASC",
        (TraceSortColumn::StartedAt, TraceSortDir::Desc) => "started_at DESC",
        (TraceSortColumn::Duration, TraceSortDir::Asc) => "duration_ms ASC",
        (TraceSortColumn::Duration, TraceSortDir::Desc) => "duration_ms DESC",
        (TraceSortColumn::SpanCount, TraceSortDir::Asc) => "span_count ASC",
        (TraceSortColumn::SpanCount, TraceSortDir::Desc) => "span_count DESC",
        (TraceSortColumn::Cost, TraceSortDir::Asc) => "total_cost_microdollars ASC",
        (TraceSortColumn::Cost, TraceSortDir::Desc) => "total_cost_microdollars DESC",
        (TraceSortColumn::Tokens, TraceSortDir::Asc) => "total_tokens ASC",
        (TraceSortColumn::Tokens, TraceSortDir::Desc) => "total_tokens DESC",
    }
}

/// Returns a list of trace summaries inside the time range.
///
/// Joins `governance_decisions`, `ai_requests`, and `plugin_usage_events` on
/// `session_id`, computing per-session aggregates. The `total_count` is the
/// count of distinct sessions matching the filter (without pagination).
fn build_trace_list_sql(sort: TraceSort) -> String {
    let order_by = order_by_clause(sort);
    format!("{TRACE_LIST_SQL_BODY} ORDER BY {order_by} LIMIT $10 OFFSET $11")
}

const TRACE_LIST_SQL_BODY: &str = r"WITH trace_to_session AS (
            -- Canonical session_id for each trace_id, used to collapse rows
            -- where governance_decisions.session_id was filled with the
            -- trace_id (data quirk in older runs).
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
            session_id,
            trace_id,
            started_at,
            ended_at,
            duration_ms,
            user_id,
            agent_id,
            agent_scope,
            model,
            provider,
            span_count,
            request_count,
            tool_call_count,
            governance_count,
            deny_count,
            total_tokens,
            input_tokens,
            output_tokens,
            total_cost_microdollars,
            total_latency_ms,
            cache_hit_any,
            top_tool,
            has_error,
            has_deny,
            total_count
        FROM counted";

pub async fn fetch_trace_list(
    pool: &PgPool,
    filter: TraceFilter<'_>,
    range: TimeRange,
    sort: TraceSort,
    limit: i64,
    offset: i64,
) -> Result<(Vec<TraceSummary>, i64), sqlx::Error> {
    let sql = build_trace_list_sql(sort);

    let rows: Vec<TraceRow> = sqlx::query_as(&sql)
        .bind(range.from)
        .bind(range.to)
        .bind(filter.user_id)
        .bind(filter.agent_id)
        .bind(filter.agent_scope)
        .bind(filter.policy)
        .bind(filter.decision)
        .bind(filter.error_only)
        .bind(filter.deny_only)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let total = rows.first().map_or(0, |r| r.total_count);
    let summaries = rows
        .into_iter()
        .map(|r| TraceSummary {
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
        })
        .collect();
    Ok((summaries, total))
}

/// Aggregate stats for the list page header (p50/p95/p99).
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct TraceStats {
    pub total_traces: i64,
    pub error_count: i64,
    pub deny_count: i64,
    pub p50_duration_ms: i64,
    pub p95_duration_ms: i64,
    pub p99_duration_ms: i64,
}

/// Per-session percentile stats over the same window as the list.
pub async fn fetch_trace_stats(pool: &PgPool, range: TimeRange) -> Result<TraceStats, sqlx::Error> {
    let row = sqlx::query!(
        r#"WITH trace_to_session AS (
            SELECT DISTINCT trace_id, session_id
            FROM ai_requests
            WHERE created_at >= $1 AND created_at < $2
              AND trace_id IS NOT NULL AND session_id IS NOT NULL
        ),
        all_sessions AS (
            SELECT session_id, created_at, NULL::text AS decision, NULL::text AS status
            FROM plugin_usage_events
            WHERE created_at >= $1 AND created_at < $2 AND session_id IS NOT NULL
            UNION ALL
            SELECT COALESCE(t.session_id, g.session_id) AS session_id,
                   g.created_at, g.decision, NULL::text
            FROM governance_decisions g
            LEFT JOIN trace_to_session t ON t.trace_id = g.session_id
            WHERE g.created_at >= $1 AND g.created_at < $2 AND g.session_id IS NOT NULL
            UNION ALL
            SELECT session_id, created_at, NULL::text, status::text
            FROM ai_requests
            WHERE created_at >= $1 AND created_at < $2 AND session_id IS NOT NULL
        ),
        per_session AS (
            SELECT
                session_id,
                EXTRACT(EPOCH FROM (MAX(created_at) - MIN(created_at))) * 1000 AS duration_ms,
                BOOL_OR(decision = 'deny') AS has_deny,
                BOOL_OR(status NOT IN ('ok','success','completed','pending') AND status IS NOT NULL)
                  AS has_error
            FROM all_sessions
            GROUP BY session_id
        )
        SELECT
            COUNT(*)::bigint                                                AS "total_traces!",
            COUNT(*) FILTER (WHERE has_error)::bigint                       AS "error_count!",
            COUNT(*) FILTER (WHERE has_deny)::bigint                        AS "deny_count!",
            COALESCE(percentile_disc(0.50) WITHIN GROUP (ORDER BY duration_ms), 0)::bigint
                                                                            AS "p50!",
            COALESCE(percentile_disc(0.95) WITHIN GROUP (ORDER BY duration_ms), 0)::bigint
                                                                            AS "p95!",
            COALESCE(percentile_disc(0.99) WITHIN GROUP (ORDER BY duration_ms), 0)::bigint
                                                                            AS "p99!"
        FROM per_session"#,
        range.from,
        range.to,
    )
    .fetch_one(pool)
    .await?;

    Ok(TraceStats {
        total_traces: row.total_traces,
        error_count: row.error_count,
        deny_count: row.deny_count,
        p50_duration_ms: row.p50,
        p95_duration_ms: row.p95,
        p99_duration_ms: row.p99,
    })
}

/// One waterfall span — normalised across the four span sources.
#[derive(Debug, Clone, Serialize)]
pub struct Span {
    pub id: String,
    pub kind: SpanKind,
    pub name: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub duration_ms: i64,
    pub status: SpanStatus,
    pub identity_label: Option<String>,
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SpanKind {
    Gateway,
    Governance,
    Tool,
    Model,
    Spawn,
}

impl SpanKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Gateway => "gateway",
            Self::Governance => "governance",
            Self::Tool => "tool",
            Self::Model => "model",
            Self::Spawn => "spawn",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SpanStatus {
    Ok,
    Deny,
    Error,
    Pending,
}

impl SpanStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Deny => "deny",
            Self::Error => "error",
            Self::Pending => "pending",
        }
    }
}

/// Resolve `id` (a `session_id` or `trace_id`) to an absolute `session_id`.
pub async fn resolve_trace_session(pool: &PgPool, id: &str) -> Result<Option<String>, sqlx::Error> {
    if let Some(row) = sqlx::query!(
        r#"SELECT session_id AS "session_id!"
           FROM ai_requests
           WHERE trace_id = $1 AND session_id IS NOT NULL
           LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(Some(row.session_id));
    }

    if let Some(row) = sqlx::query!(
        r#"SELECT session_id AS "session_id!"
           FROM governance_decisions
           WHERE session_id = $1
           LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(Some(row.session_id));
    }

    if let Some(row) = sqlx::query!(
        r#"SELECT session_id AS "session_id!"
           FROM ai_requests
           WHERE session_id = $1
           LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(Some(row.session_id));
    }

    Ok(None)
}

/// Fetches all spans for one trace (`session_id`), ordered by start time.
async fn fetch_governance_spans(pool: &PgPool, session_id: &str) -> Result<Vec<Span>, sqlx::Error> {
    let decisions = sqlx::query!(
        r#"SELECT
            id              AS "id!",
            tool_name       AS "tool_name!",
            policy          AS "policy!",
            decision        AS "decision!",
            agent_id,
            agent_scope,
            user_id         AS "user_id!",
            created_at      AS "created_at!"
        FROM governance_decisions
        WHERE session_id = $1
           OR session_id IN (
               SELECT DISTINCT trace_id FROM ai_requests
               WHERE session_id = $1 AND trace_id IS NOT NULL)
        ORDER BY created_at ASC"#,
        session_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(decisions
        .into_iter()
        .map(|d| {
            let started = d.created_at;
            let status = if d.decision == "deny" {
                SpanStatus::Deny
            } else {
                SpanStatus::Ok
            };
            Span {
                id: d.id.clone(),
                kind: SpanKind::Governance,
                name: format!("{} / {}", d.policy, d.tool_name),
                started_at: started,
                ended_at: started,
                duration_ms: 0,
                status,
                identity_label: Some(format_identity(
                    Some(d.user_id.as_str()),
                    d.agent_id.as_deref(),
                    d.agent_scope.as_deref(),
                )),
                raw: serde_json::json!({
                    "policy": d.policy,
                    "decision": d.decision,
                    "tool_name": d.tool_name,
                    "agent_id": d.agent_id,
                }),
            }
        })
        .collect())
}

async fn fetch_request_spans(pool: &PgPool, session_id: &str) -> Result<Vec<Span>, sqlx::Error> {
    let requests = sqlx::query!(
        r#"SELECT
            id              AS "id!",
            request_id      AS "request_id!",
            provider        AS "provider!",
            model           AS "model!",
            status          AS "status!",
            latency_ms,
            created_at      AS "created_at!",
            completed_at,
            user_id         AS "user_id!"
        FROM ai_requests
        WHERE session_id = $1
        ORDER BY created_at ASC"#,
        session_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(requests
        .into_iter()
        .map(|r| {
            let started = r.created_at;
            let ended = r.completed_at.unwrap_or_else(|| {
                started
                    + chrono::Duration::milliseconds(i64::from(r.latency_ms.unwrap_or(0)).max(0))
            });
            let dur = (ended - started).num_milliseconds().max(0);
            let status = match r.status.as_str() {
                "ok" | "success" | "completed" => SpanStatus::Ok,
                "pending" => SpanStatus::Pending,
                _ => SpanStatus::Error,
            };
            Span {
                id: r.id.clone(),
                kind: SpanKind::Model,
                name: format!("{}/{}", r.provider, r.model),
                started_at: started,
                ended_at: ended,
                duration_ms: dur,
                status,
                identity_label: Some(format_identity(Some(r.user_id.as_str()), None, None)),
                raw: serde_json::json!({
                    "request_id": r.request_id,
                    "provider": r.provider,
                    "model": r.model,
                    "status": r.status,
                    "latency_ms": r.latency_ms,
                }),
            }
        })
        .collect())
}

async fn fetch_event_spans(pool: &PgPool, session_id: &str) -> Result<Vec<Span>, sqlx::Error> {
    let events = sqlx::query!(
        r#"SELECT
            id              AS "id!",
            event_type      AS "event_type!",
            tool_name,
            user_id         AS "user_id!",
            created_at      AS "created_at!"
        FROM plugin_usage_events
        WHERE session_id = $1
        ORDER BY created_at ASC"#,
        session_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(events
        .into_iter()
        .map(|e| {
            let started = e.created_at;
            let kind = if e.event_type.contains("Spawn") {
                SpanKind::Spawn
            } else {
                SpanKind::Tool
            };
            let name = e.tool_name.clone().unwrap_or_else(|| e.event_type.clone());
            let status = if e.event_type.contains("Failure") || e.event_type.contains("Error") {
                SpanStatus::Error
            } else {
                SpanStatus::Ok
            };
            Span {
                id: e.id.clone(),
                kind,
                name,
                started_at: started,
                ended_at: started,
                duration_ms: 0,
                status,
                identity_label: Some(format_identity(Some(e.user_id.as_str()), None, None)),
                raw: serde_json::json!({
                    "event_type": e.event_type,
                    "tool_name": e.tool_name,
                }),
            }
        })
        .collect())
}

pub async fn fetch_trace_spans(pool: &PgPool, session_id: &str) -> Result<Vec<Span>, sqlx::Error> {
    let mut spans = fetch_governance_spans(pool, session_id).await?;
    spans.extend(fetch_request_spans(pool, session_id).await?);
    spans.extend(fetch_event_spans(pool, session_id).await?);
    spans.sort_by_key(|s| s.started_at);
    Ok(spans)
}

fn format_identity(user: Option<&str>, agent: Option<&str>, scope: Option<&str>) -> String {
    let user_part = user.unwrap_or("?");
    match (agent, scope) {
        (Some(a), Some(s)) => format!("{user_part} · {a} ({s})"),
        (Some(a), None) => format!("{user_part} · {a}"),
        _ => user_part.to_string(),
    }
}
