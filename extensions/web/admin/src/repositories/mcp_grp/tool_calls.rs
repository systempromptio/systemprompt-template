//! Tool-call read models for the MCP Tools page and per-tool drill page.
//!
//! Sources:
//! - `plugin_usage_events` (tool invocations — `event_type ILIKE '%ToolUse%'`)
//! - `governance_decisions` (verdict for the same `session_id` + `tool_name`)
//! - `ai_requests` (parent gateway request — for `trace_id` surfacing)

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

use crate::repositories::governance_grp::time_range::TimeRange;

#[derive(Debug, Clone, Default)]
pub struct ToolCallFilter {
    pub tool_name: Option<String>,
    pub user_id: Option<String>,
    pub agent_scope: Option<String>,
    pub plugin_id: Option<String>,
    pub decision: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum ToolSortColumn {
    CreatedAt,
    Bytes,
    Latency,
}

#[derive(Debug, Clone, Copy)]
pub enum SortDir {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy)]
pub struct ToolSortSpec {
    pub column: ToolSortColumn,
    pub dir: SortDir,
}

impl Default for ToolSortSpec {
    fn default() -> Self {
        Self {
            column: ToolSortColumn::CreatedAt,
            dir: SortDir::Desc,
        }
    }
}

impl ToolSortSpec {
    const fn order_by(self) -> &'static str {
        match (self.column, self.dir) {
            (ToolSortColumn::CreatedAt, SortDir::Asc) => "e.created_at ASC",
            (ToolSortColumn::CreatedAt, SortDir::Desc) => "e.created_at DESC",
            (ToolSortColumn::Bytes, SortDir::Asc) => {
                "(COALESCE(e.content_input_bytes,0)+COALESCE(e.content_output_bytes,0)) ASC"
            }
            (ToolSortColumn::Bytes, SortDir::Desc) => {
                "(COALESCE(e.content_input_bytes,0)+COALESCE(e.content_output_bytes,0)) DESC"
            }
            (ToolSortColumn::Latency, SortDir::Asc) => "ar_latency_ms ASC NULLS LAST",
            (ToolSortColumn::Latency, SortDir::Desc) => "ar_latency_ms DESC NULLS LAST",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolCallRow {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub event_type: String,
    pub tool_name: Option<String>,
    pub plugin_id: Option<String>,
    pub user_id: String,
    pub session_id: String,
    pub agent_id: Option<String>,
    pub agent_scope: Option<String>,
    pub content_input_bytes: i64,
    pub content_output_bytes: i64,
    pub decision: Option<String>,
    pub policy: Option<String>,
    pub reason: Option<String>,
    pub trace_id: Option<String>,
    pub ar_latency_ms: Option<i32>,
    pub metadata: serde_json::Value,
}

/// Page through tool-call events with their governance verdict and parent
/// request `trace_id`. Filter / sort / search supported.
pub async fn fetch_tool_calls_paged(
    pool: &PgPool,
    filter: &ToolCallFilter,
    range: TimeRange,
    sort: ToolSortSpec,
    limit: i64,
    offset: i64,
) -> Result<(Vec<ToolCallRow>, i64), sqlx::Error> {
    let search_pattern = filter
        .search
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{s}%"));

    let order = sort.order_by();
    let sql = format!(
        r"WITH joined AS (
            SELECT
                e.id, e.created_at, e.event_type, e.tool_name, e.plugin_id,
                e.user_id, e.session_id,
                COALESCE(e.content_input_bytes, 0)::bigint AS content_input_bytes,
                COALESCE(e.content_output_bytes, 0)::bigint AS content_output_bytes,
                COALESCE(e.metadata, '{{}}'::jsonb) AS metadata,
                gd.decision, gd.policy, gd.reason, gd.agent_id, gd.agent_scope,
                ar.trace_id AS trace_id,
                ar.latency_ms AS ar_latency_ms
            FROM plugin_usage_events e
            LEFT JOIN LATERAL (
                SELECT decision, policy, reason, agent_id, agent_scope
                FROM governance_decisions g
                WHERE g.session_id = e.session_id
                  AND (g.tool_name = e.tool_name OR e.tool_name IS NULL)
                ORDER BY g.created_at DESC
                LIMIT 1
            ) gd ON TRUE
            LEFT JOIN LATERAL (
                SELECT trace_id, latency_ms
                FROM ai_requests ar2
                WHERE ar2.session_id = e.session_id
                ORDER BY ar2.created_at DESC
                LIMIT 1
            ) ar ON TRUE
            WHERE e.created_at >= $1 AND e.created_at < $2
              AND e.event_type ILIKE '%ToolUse%'
              AND ($3::text IS NULL OR e.tool_name = $3)
              AND ($4::text IS NULL OR e.user_id = $4)
              AND ($5::text IS NULL OR gd.agent_scope = $5)
              AND ($6::text IS NULL OR e.plugin_id = $6)
              AND ($7::text IS NULL OR gd.decision = $7)
              AND ($8::text IS NULL
                   OR e.user_id ILIKE $8
                   OR COALESCE(e.tool_name, '') ILIKE $8
                   OR COALESCE(e.plugin_id, '') ILIKE $8
                   OR COALESCE(gd.reason, '') ILIKE $8)
        )
        SELECT
            id, created_at, event_type, tool_name, plugin_id,
            user_id, session_id, agent_id, agent_scope,
            content_input_bytes, content_output_bytes,
            decision, policy, reason, trace_id, ar_latency_ms, metadata,
            (SELECT COUNT(*) FROM joined)::bigint AS total_count
        FROM joined
        ORDER BY {order}
        LIMIT $9 OFFSET $10",
    );

    // Why: {order} interpolates ORDER BY built from a closed sort enum.
    let rows = sqlx::query_as::<_, ToolCallRowWithTotal>(&sql)
        .bind(range.from)
        .bind(range.to)
        .bind(filter.tool_name.as_deref())
        .bind(filter.user_id.as_deref())
        .bind(filter.agent_scope.as_deref())
        .bind(filter.plugin_id.as_deref())
        .bind(filter.decision.as_deref())
        .bind(search_pattern.as_deref())
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let total = rows.first().map_or(0, |r| r.total_count);
    let out = rows
        .into_iter()
        .map(|r| ToolCallRow {
            id: r.id,
            created_at: r.created_at,
            event_type: r.event_type,
            tool_name: r.tool_name,
            plugin_id: r.plugin_id,
            user_id: r.user_id,
            session_id: r.session_id,
            agent_id: r.agent_id,
            agent_scope: r.agent_scope,
            content_input_bytes: r.content_input_bytes,
            content_output_bytes: r.content_output_bytes,
            decision: r.decision,
            policy: r.policy,
            reason: r.reason,
            trace_id: r.trace_id,
            ar_latency_ms: r.ar_latency_ms,
            metadata: r.metadata,
        })
        .collect();
    Ok((out, total))
}

#[derive(sqlx::FromRow)]
struct ToolCallRowWithTotal {
    id: String,
    created_at: DateTime<Utc>,
    event_type: String,
    tool_name: Option<String>,
    plugin_id: Option<String>,
    user_id: String,
    session_id: String,
    agent_id: Option<String>,
    agent_scope: Option<String>,
    content_input_bytes: i64,
    content_output_bytes: i64,
    decision: Option<String>,
    policy: Option<String>,
    reason: Option<String>,
    trace_id: Option<String>,
    ar_latency_ms: Option<i32>,
    metadata: serde_json::Value,
    total_count: i64,
}

/// Per-tool aggregates used by the per-tool drill page.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ToolDetailStats {
    pub tool_name: String,
    pub total_calls: i64,
    pub allow_count: i64,
    pub deny_count: i64,
    pub distinct_users: i64,
    pub distinct_agents: i64,
    pub total_bytes_in: i64,
    pub total_bytes_out: i64,
}

pub async fn fetch_tool_detail_stats(
    pool: &PgPool,
    tool_name: &str,
    range: TimeRange,
) -> Result<ToolDetailStats, sqlx::Error> {
    let row = sqlx::query!(
        r#"WITH calls AS (
            SELECT e.id, e.user_id, e.session_id,
                   COALESCE(e.content_input_bytes, 0)::bigint AS content_input_bytes,
                   COALESCE(e.content_output_bytes, 0)::bigint AS content_output_bytes
            FROM plugin_usage_events e
            WHERE e.created_at >= $1 AND e.created_at < $2
              AND e.event_type ILIKE '%ToolUse%'
              AND e.tool_name = $3
        ),
        verdicts AS (
            SELECT DISTINCT c.id,
                   COALESCE((
                     SELECT g.decision FROM governance_decisions g
                     WHERE g.session_id = c.session_id AND g.tool_name = $3
                     ORDER BY g.created_at DESC LIMIT 1
                   ), 'unknown') AS decision,
                   (SELECT g.agent_id FROM governance_decisions g
                    WHERE g.session_id = c.session_id AND g.tool_name = $3
                    ORDER BY g.created_at DESC LIMIT 1) AS agent_id
            FROM calls c
        )
        SELECT
            COUNT(*)::bigint AS "total!",
            COUNT(*) FILTER (WHERE v.decision = 'allow')::bigint AS "allow!",
            COUNT(*) FILTER (WHERE v.decision = 'deny')::bigint AS "deny!",
            COUNT(DISTINCT c.user_id)::bigint AS "distinct_users!",
            COUNT(DISTINCT v.agent_id) FILTER (WHERE v.agent_id IS NOT NULL)::bigint
                AS "distinct_agents!",
            COALESCE(SUM(c.content_input_bytes), 0)::bigint AS "bytes_in!",
            COALESCE(SUM(c.content_output_bytes), 0)::bigint AS "bytes_out!"
        FROM calls c
        LEFT JOIN verdicts v ON v.id = c.id"#,
        range.from,
        range.to,
        tool_name,
    )
    .fetch_one(pool)
    .await?;

    Ok(ToolDetailStats {
        tool_name: tool_name.to_string(),
        total_calls: row.total,
        allow_count: row.allow,
        deny_count: row.deny,
        distinct_users: row.distinct_users,
        distinct_agents: row.distinct_agents,
        total_bytes_in: row.bytes_in,
        total_bytes_out: row.bytes_out,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolDenyReason {
    pub reason: String,
    pub policy: String,
    pub count: i64,
}

pub async fn fetch_tool_deny_reasons(
    pool: &PgPool,
    tool_name: &str,
    range: TimeRange,
    limit: i64,
) -> Result<Vec<ToolDenyReason>, sqlx::Error> {
    sqlx::query_as!(
        ToolDenyReason,
        r#"SELECT
            reason as "reason!",
            policy as "policy!",
            COUNT(*)::bigint AS "count!"
          FROM governance_decisions
          WHERE created_at >= $1 AND created_at < $2
            AND tool_name = $3
            AND decision = 'deny'
          GROUP BY reason, policy
          ORDER BY COUNT(*) DESC
          LIMIT $4"#,
        range.from,
        range.to,
        tool_name,
        limit,
    )
    .fetch_all(pool)
    .await
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolTopActor {
    pub identity_id: String,
    pub label: String,
    pub deny_count: i64,
    pub total_count: i64,
}

#[derive(Debug, Clone, Copy)]
pub enum ToolActorGroup {
    User,
    Agent,
}

pub async fn fetch_tool_top_actors(
    pool: &PgPool,
    tool_name: &str,
    range: TimeRange,
    group: ToolActorGroup,
    limit: i64,
) -> Result<Vec<ToolTopActor>, sqlx::Error> {
    match group {
        ToolActorGroup::User => {
            sqlx::query_as!(
                ToolTopActor,
                r#"SELECT
                g.user_id as "identity_id!",
                COALESCE(u.display_name, u.full_name, u.name, u.email, g.user_id)
                    as "label!",
                COUNT(*) FILTER (WHERE g.decision = 'deny')::bigint as "deny_count!",
                COUNT(*)::bigint as "total_count!"
              FROM governance_decisions g
              LEFT JOIN users u ON u.id = g.user_id
              WHERE g.created_at >= $1 AND g.created_at < $2
                AND g.tool_name = $3
              GROUP BY g.user_id, u.display_name, u.full_name, u.name, u.email
              ORDER BY COUNT(*) FILTER (WHERE g.decision = 'deny') DESC, COUNT(*) DESC
              LIMIT $4"#,
                range.from,
                range.to,
                tool_name,
                limit,
            )
            .fetch_all(pool)
            .await
        }
        ToolActorGroup::Agent => {
            sqlx::query_as!(
                ToolTopActor,
                r#"SELECT
                COALESCE(g.agent_id, '')   as "identity_id!",
                COALESCE(g.agent_id, '—')  as "label!",
                COUNT(*) FILTER (WHERE g.decision = 'deny')::bigint as "deny_count!",
                COUNT(*)::bigint as "total_count!"
              FROM governance_decisions g
              WHERE g.created_at >= $1 AND g.created_at < $2
                AND g.tool_name = $3
                AND g.agent_id IS NOT NULL
              GROUP BY g.agent_id
              ORDER BY COUNT(*) FILTER (WHERE g.decision = 'deny') DESC, COUNT(*) DESC
              LIMIT $4"#,
                range.from,
                range.to,
                tool_name,
                limit,
            )
            .fetch_all(pool)
            .await
        }
    }
}
