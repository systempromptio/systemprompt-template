//! Per-entity health rollups for `/admin/overview/services`.
//!
//! Aggregates request/error/latency over the four "service" entity classes
//! shown on the page:
//! - **Agents** (governance_decisions, plugin_usage_events keyed by agent_id)
//! - **MCP servers** (mcp_tool_executions keyed by server_name)
//! - **Gateway** (ai_requests, single logical entity)
//!
//! External agents have no runtime traffic table — they get an inventory-only
//! view from `external_agents_grp::list_external_agents`.

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::repositories::governance_grp::time_range::TimeRange;

#[derive(Debug, Clone, Default)]
pub struct AgentHealthRow {
    pub agent_id: String,
    pub allowed: i64,
    pub denied: i64,
    pub last_denied_at: Option<DateTime<Utc>>,
}

pub async fn fetch_agent_health(
    pool: &PgPool,
    range: TimeRange,
) -> Result<Vec<AgentHealthRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT
            agent_id AS "agent_id!",
            COUNT(*) FILTER (WHERE decision = 'allow')::bigint AS "allowed!",
            COUNT(*) FILTER (WHERE decision = 'deny')::bigint  AS "denied!",
            MAX(created_at) FILTER (WHERE decision = 'deny')   AS "last_denied_at"
          FROM governance_decisions
          WHERE created_at >= $1 AND created_at < $2
            AND agent_id IS NOT NULL
          GROUP BY agent_id"#,
        range.from,
        range.to,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| AgentHealthRow {
            agent_id: r.agent_id,
            allowed: r.allowed,
            denied: r.denied,
            last_denied_at: r.last_denied_at,
        })
        .collect())
}

#[derive(Debug, Clone, Default)]
pub struct McpServerHealthRow {
    pub server_name: String,
    pub calls: i64,
    pub errors: i64,
    pub avg_latency_ms: f64,
    pub last_error_at: Option<DateTime<Utc>>,
}

pub async fn fetch_mcp_server_health(
    pool: &PgPool,
    range: TimeRange,
) -> Result<Vec<McpServerHealthRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT
            server_name AS "server_name!",
            COUNT(*)::bigint AS "calls!",
            COUNT(*) FILTER (WHERE status = 'failed')::bigint AS "errors!",
            COALESCE(AVG(execution_time_ms), 0)::float8 AS "avg_latency!",
            MAX(created_at) FILTER (WHERE status = 'failed') AS "last_error_at"
          FROM mcp_tool_executions
          WHERE created_at >= $1 AND created_at < $2
          GROUP BY server_name"#,
        range.from,
        range.to,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| McpServerHealthRow {
            server_name: r.server_name,
            calls: r.calls,
            errors: r.errors,
            avg_latency_ms: r.avg_latency,
            last_error_at: r.last_error_at,
        })
        .collect())
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GatewayHealth {
    pub requests: i64,
    pub errors: i64,
    pub avg_latency_ms: f64,
}

pub async fn fetch_gateway_health(
    pool: &PgPool,
    range: TimeRange,
) -> Result<GatewayHealth, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT
            COUNT(*)::bigint AS "requests!",
            COUNT(*) FILTER (WHERE status NOT IN ('completed','pending','streaming'))::bigint
                AS "errors!",
            COALESCE(AVG(latency_ms), 0)::float8 AS "avg_latency!"
          FROM ai_requests
          WHERE created_at >= $1 AND created_at < $2"#,
        range.from,
        range.to,
    )
    .fetch_one(pool)
    .await?;

    Ok(GatewayHealth {
        requests: row.requests,
        errors: row.errors,
        avg_latency_ms: row.avg_latency,
    })
}
