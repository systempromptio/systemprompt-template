//! The five decision-relevant totals above the request log.
//!
//! Deliberately computed over the *filtered* set rather than the raw window:
//! the numbers name what the table below them holds, so narrowing to one model
//! or one project moves them. `unattributed` is an explicit column rather than
//! a silently dropped row — a request from someone `user_scope_defaults` has
//! no primary group or project for still counts, and says so.

use sqlx::PgPool;
use systemprompt::identifiers::{AgentId, UserId};

use super::RequestFilter;
use crate::util::time_range::TimeRange;

/// Window totals for the request log, over the same predicate the list uses.
#[derive(Debug, Clone, Copy, Default)]
pub struct RequestKpis {
    pub total: i64,
    pub failed: i64,
    pub rejected: i64,
    pub cost_microdollars: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub tool_calls: i64,
    pub denied: i64,
    pub unattributed: i64,
}

pub async fn get_request_kpis(
    pool: &PgPool,
    filter: &RequestFilter,
    range: TimeRange,
) -> Result<RequestKpis, sqlx::Error> {
    let search_pattern = filter
        .search
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{s}%"));

    let row = sqlx::query_file!(
        "src/repositories/analytics/requests/kpis.sql",
        range.from,
        range.to,
        filter.user_id.as_ref().map(UserId::as_str),
        filter.agent_id.as_ref().map(AgentId::as_str),
        filter.model.as_deref(),
        filter.provider.as_deref(),
        filter.status.as_deref(),
        filter.tool.as_deref(),
        search_pattern,
        filter.scope.as_sql(),
        filter.group.as_deref(),
        filter.project.as_deref(),
    )
    .fetch_one(pool)
    .await?;

    Ok(RequestKpis {
        total: row.total,
        failed: row.failed,
        rejected: row.rejected,
        cost_microdollars: row.cost_microdollars,
        input_tokens: row.input_tokens,
        output_tokens: row.output_tokens,
        p50_latency_ms: row.p50_latency_ms,
        p95_latency_ms: row.p95_latency_ms,
        tool_calls: row.tool_calls,
        denied: row.denied,
        unattributed: row.unattributed,
    })
}
