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
    pub model: Option<String>,
    pub span_count: i64,
    pub has_error: bool,
    pub has_deny: bool,
}

/// Raw row tuple returned by the dynamic `fetch_trace_list` query.
///
/// Tuple fields, in order: `session_id`, `trace_id`, `started_at`, `ended_at`,
/// `duration_ms`, `user_id`, `agent_id`, `model`, `span_count`, `has_error`,
/// `has_deny`, `total_count`.
type TraceRowTuple = (
    String,
    Option<String>,
    DateTime<Utc>,
    DateTime<Utc>,
    i64,
    Option<String>,
    Option<String>,
    Option<String>,
    i64,
    bool,
    bool,
    i64,
);

#[derive(Debug, Clone, Copy, Default)]
pub struct TraceFilter<'a> {
    pub user_id: Option<&'a str>,
    pub agent_id: Option<&'a str>,
    pub agent_scope: Option<&'a str>,
    pub error_only: bool,
    pub deny_only: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum TraceSortColumn {
    StartedAt,
    Duration,
    SpanCount,
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
    }
}

/// Returns a list of trace summaries inside the time range.
///
/// Joins `governance_decisions`, `ai_requests`, and `plugin_usage_events` on
/// `session_id`, computing per-session aggregates. The `total_count` is the
/// count of distinct sessions matching the filter (without pagination).
fn build_trace_list_sql(sort: TraceSort) -> String {
    let order_by = order_by_clause(sort);
    format!(
        r"WITH all_sessions AS (
            SELECT session_id, user_id, agent_id, agent_scope,
                   created_at, decision
            FROM governance_decisions
            WHERE created_at >= $1 AND created_at < $2
              AND session_id IS NOT NULL
            UNION ALL
            SELECT session_id, user_id, NULL::text AS agent_id, NULL::text AS agent_scope,
                   created_at, NULL::text AS decision
            FROM ai_requests
            WHERE created_at >= $1 AND created_at < $2
              AND session_id IS NOT NULL
            UNION ALL
            SELECT session_id, user_id, NULL::text AS agent_id, NULL::text AS agent_scope,
                   created_at, NULL::text AS decision
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
                COUNT(*) FILTER (WHERE decision = 'deny')::bigint > 0 AS has_deny
            FROM all_sessions
            GROUP BY session_id
        ),
        ai_meta AS (
            SELECT
                session_id,
                (ARRAY_AGG(trace_id ORDER BY created_at DESC))[1]   AS trace_id,
                (ARRAY_AGG(model    ORDER BY created_at DESC))[1]   AS model,
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
                EXTRACT(EPOCH FROM (p.ended_at - p.started_at)) * 1000 AS duration_ms,
                p.span_count,
                p.has_deny,
                a.trace_id,
                a.model,
                COALESCE(a.has_error, false) AS has_error
            FROM per_session p
            LEFT JOIN ai_meta a ON a.session_id = p.session_id
        ),
        filtered AS (
            SELECT * FROM joined
            WHERE ($3::text IS NULL OR user_id   = $3)
              AND ($4::text IS NULL OR agent_id  = $4)
              AND ($5::text IS NULL OR agent_scope = $5)
              AND (NOT $6 OR has_error = true)
              AND (NOT $7 OR has_deny  = true)
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
            duration_ms::bigint AS duration_ms,
            user_id,
            agent_id,
            model,
            span_count,
            has_error,
            has_deny,
            total_count
        FROM counted
        ORDER BY {order_by}
        LIMIT $8 OFFSET $9"
    )
}

pub async fn fetch_trace_list(
    pool: &PgPool,
    filter: TraceFilter<'_>,
    range: TimeRange,
    sort: TraceSort,
    limit: i64,
    offset: i64,
) -> Result<(Vec<TraceSummary>, i64), sqlx::Error> {
    let sql = build_trace_list_sql(sort);

    let rows: Vec<TraceRowTuple> = sqlx::query_as(&sql)
        .bind(range.from)
        .bind(range.to)
        .bind(filter.user_id)
        .bind(filter.agent_id)
        .bind(filter.agent_scope)
        .bind(filter.error_only)
        .bind(filter.deny_only)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let total = rows.first().map_or(0, |r| r.11);
    let summaries = rows
        .into_iter()
        .map(|r| TraceSummary {
            session_id: r.0,
            trace_id: r.1,
            started_at: r.2,
            ended_at: r.3,
            duration_ms: r.4,
            user_id: r.5,
            agent_id: r.6,
            model: r.7,
            span_count: r.8,
            has_error: r.9,
            has_deny: r.10,
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
pub async fn fetch_trace_stats(
    pool: &PgPool,
    range: TimeRange,
) -> Result<TraceStats, sqlx::Error> {
    let row = sqlx::query!(
        r#"WITH all_sessions AS (
            SELECT session_id, created_at, NULL::text AS decision, NULL::text AS status
            FROM plugin_usage_events
            WHERE created_at >= $1 AND created_at < $2 AND session_id IS NOT NULL
            UNION ALL
            SELECT session_id, created_at, decision, NULL::text
            FROM governance_decisions
            WHERE created_at >= $1 AND created_at < $2 AND session_id IS NOT NULL
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
pub async fn resolve_trace_session(
    pool: &PgPool,
    id: &str,
) -> Result<Option<String>, sqlx::Error> {
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
        ORDER BY created_at ASC"#,
        session_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(decisions
        .into_iter()
        .map(|d| {
            // governance_decisions has no latency column; treat decisions as point-in-time spans.
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
            let name = e
                .tool_name
                .clone()
                .unwrap_or_else(|| e.event_type.clone());
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

pub async fn fetch_trace_spans(
    pool: &PgPool,
    session_id: &str,
) -> Result<Vec<Span>, sqlx::Error> {
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
