//! `/admin/mcp/tools` — Tool Calls.
//!
//! Keeps the existing success-rate / errors / by-agent panels, then appends a
//! filterable paged table of every tool invocation joined to its governance
//! verdict and parent gateway request. Each table row carries `data-chain-id`
//! so the chain-drawer opens the full envelope on click.

use std::sync::Arc;

use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;

use crate::repositories;
use crate::repositories::analytics_grp::mcp_tools::{
    list_recent_tool_executions, list_tools_by_agent, RecentToolExecution, ToolByAgent,
};
use crate::repositories::governance_grp::time_range::{parse_time_range, TimeRangeQuery};
use crate::repositories::mcp_grp::tool_calls::{
    fetch_tool_calls_paged, ToolCallFilter, ToolCallRow, ToolSortSpec,
};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

const PAGE_SIZE: i64 = 50;

#[derive(Debug, Deserialize)]
pub struct ToolsQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub preset: Option<String>,
    pub tool_name: Option<String>,
    pub user_id: Option<String>,
    pub agent_scope: Option<String>,
    pub plugin_id: Option<String>,
    pub decision: Option<String>,
    pub q: Option<String>,
    pub page: Option<i64>,
}

pub async fn mcp_tools_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<ToolsQuery>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let range = parse_time_range(&TimeRangeQuery {
        from: query.from.clone(),
        to: query.to.clone(),
        preset: query.preset.clone(),
    });
    let filter = filter_from_query(&query);
    let sort = ToolSortSpec::default();
    let page = query.page.unwrap_or(0).max(0);
    let offset = page * PAGE_SIZE;

    let (success_rates_res, errors_res, by_agent_res, recent_res, paged) = tokio::join!(
        repositories::dashboard_queries::fetch_tool_success_rates(&pool),
        repositories::dashboard_queries::fetch_recent_mcp_errors(&pool),
        fetch_tools_by_agent(&pool),
        fetch_recent_tool_executions(&pool),
        fetch_tool_calls_paged(&pool, &filter, range, sort, PAGE_SIZE, offset),
    );

    let success_rates = success_rates_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch tool success rates");
        vec![]
    });
    let errors = errors_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch recent MCP errors");
        vec![]
    });
    let by_agent = by_agent_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch tools by agent");
        vec![]
    });
    let recent = recent_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch recent tool executions");
        vec![]
    });
    let (rows, total_count) = paged.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_tool_calls_paged failed");
        (Vec::new(), 0)
    });

    let total_pages = if total_count == 0 {
        1
    } else {
        (total_count + PAGE_SIZE - 1) / PAGE_SIZE
    };
    let total_calls: i64 = success_rates.iter().map(|r| r.total).sum();
    let total_failures: i64 = success_rates.iter().map(|r| r.failures).sum();

    let data = json!({
        "page": "mcp-tools",
        "title": "Tool Calls",
        "cli_command": "systemprompt infra logs tools list --limit 10",
        "demo_script": "demo/mcp/03-mcp-tool-execution.sh",
        "page_stats": [
            {"key": "calls", "value": total_calls, "label": "Tool calls"},
            {"key": "failures", "value": total_failures, "label": "Failures"},
            {"key": "tools", "value": success_rates.len(), "label": "Distinct tools"},
        ],
        "success_rates": success_rates.iter().map(|r| json!({
            "tool_name": r.tool_name,
            "total": r.total,
            "successes": r.successes,
            "failures": r.failures,
            "success_pct": r.success_pct,
            "detail_url": format!("/admin/mcp/tools/{}", r.tool_name),
        })).collect::<Vec<_>>(),
        "has_success_rates": !success_rates.is_empty(),
        "recent_errors": errors.iter().map(|e| json!({
            "tool_name": e.tool_name,
            "created_at": e.created_at,
        })).collect::<Vec<_>>(),
        "has_recent_errors": !errors.is_empty(),
        "tools_by_agent": by_agent.iter().map(|r| json!({
            "agent_id": r.agent_id,
            "tool_name": r.tool_name,
            "usage_count": r.usage_count,
        })).collect::<Vec<_>>(),
        "has_tools_by_agent": !by_agent.is_empty(),
        "recent_executions": recent.iter().map(|r| json!({
            "tool_name": r.tool_name,
            "status": r.status,
            "is_failed": r.status == "failed",
            "user_id": r.user_id,
            "created_at": r.created_at,
        })).collect::<Vec<_>>(),
        "has_recent_executions": !recent.is_empty(),
        "time_range": time_range_context(&query, &range),
        "tool_calls": rows.iter().map(tool_call_to_json).collect::<Vec<_>>(),
        "has_tool_calls": !rows.is_empty(),
        "total_count": total_count,
        "search_query": query.q.clone().unwrap_or_default(),
        "pagination": build_pagination(&query, page, total_pages),
        "filter": {
            "tool_name": query.tool_name.clone().unwrap_or_default(),
            "decision": query.decision.clone().unwrap_or_default(),
        },
    });
    super::render_page(&engine, "mcp-tools", &data, &user_ctx, &mkt_ctx)
}

async fn fetch_recent_tool_executions(
    pool: &PgPool,
) -> Result<Vec<RecentToolExecution>, sqlx::Error> {
    list_recent_tool_executions(pool).await
}

async fn fetch_tools_by_agent(pool: &PgPool) -> Result<Vec<ToolByAgent>, sqlx::Error> {
    list_tools_by_agent(pool).await
}

fn filter_from_query(query: &ToolsQuery) -> ToolCallFilter {
    ToolCallFilter {
        tool_name: empty_to_none(query.tool_name.as_ref()),
        user_id: empty_to_none(query.user_id.as_ref()),
        agent_scope: empty_to_none(query.agent_scope.as_ref()),
        plugin_id: empty_to_none(query.plugin_id.as_ref()),
        decision: empty_to_none(query.decision.as_ref()),
        search: empty_to_none(query.q.as_ref()),
    }
}

fn empty_to_none(v: Option<&String>) -> Option<String> {
    v.map(String::as_str)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
}

fn tool_call_to_json(r: &ToolCallRow) -> serde_json::Value {
    json!({
        "id": r.id,
        "trace_id": r.trace_id,
        "session_id": r.session_id,
        "tool_name": r.tool_name,
        "plugin_id": r.plugin_id,
        "user_id": r.user_id,
        "agent_id": r.agent_id,
        "agent_scope": r.agent_scope,
        "event_type": r.event_type,
        "content_input_bytes": r.content_input_bytes,
        "content_output_bytes": r.content_output_bytes,
        "bytes_total": r.content_input_bytes + r.content_output_bytes,
        "decision": r.decision,
        "is_denied": r.decision.as_deref() == Some("deny"),
        "is_allowed": r.decision.as_deref() == Some("allow"),
        "policy": r.policy,
        "rule_name": extract_rule_name(&r.metadata),
        "reason": r.reason,
        "latency_ms": r.ar_latency_ms,
        "created_at": r.created_at.to_rfc3339(),
        "created_at_local": r
            .created_at
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
        "metadata": r.metadata,
        "metadata_present": !r.metadata.is_null()
            && r.metadata.as_object().is_some_and(|m| !m.is_empty()),
        "tool_detail_url": r.tool_name.as_deref().map(|t| format!(
            "/admin/mcp/tools/{}",
            urlencoding::encode(t)
        )),
    })
}

fn extract_rule_name(metadata: &serde_json::Value) -> Option<String> {
    metadata
        .get("rule_name")
        .or_else(|| metadata.get("rule"))
        .and_then(|v| v.as_str())
        .map(ToString::to_string)
}

fn time_range_context(
    query: &ToolsQuery,
    range: &repositories::governance_grp::time_range::TimeRange,
) -> serde_json::Value {
    let preset = query.preset.clone().unwrap_or_else(|| {
        if query.from.is_some() && query.to.is_some() {
            "custom".to_string()
        } else {
            "24h".to_string()
        }
    });
    json!({
        "preset": preset,
        "from": range.from.to_rfc3339(),
        "to": range.to.to_rfc3339(),
        "base_url": "/admin/mcp/tools",
        "query": "",
    })
}

fn build_pagination(query: &ToolsQuery, page: i64, total_pages: i64) -> serde_json::Value {
    let mut params = Vec::new();
    if let Some(p) = query.preset.as_deref().filter(|s| !s.is_empty()) {
        params.push(format!("preset={}", urlencoding::encode(p)));
    }
    if let Some(f) = query.from.as_deref().filter(|s| !s.is_empty()) {
        params.push(format!("from={}", urlencoding::encode(f)));
    }
    if let Some(t) = query.to.as_deref().filter(|s| !s.is_empty()) {
        params.push(format!("to={}", urlencoding::encode(t)));
    }
    if let Some(t) = query.tool_name.as_deref().filter(|s| !s.is_empty()) {
        params.push(format!("tool_name={}", urlencoding::encode(t)));
    }
    if let Some(d) = query.decision.as_deref().filter(|s| !s.is_empty()) {
        params.push(format!("decision={}", urlencoding::encode(d)));
    }
    if let Some(q) = query.q.as_deref().filter(|s| !s.is_empty()) {
        params.push(format!("q={}", urlencoding::encode(q)));
    }
    let qs = params.join("&");
    let prefix = if qs.is_empty() {
        "/admin/mcp/tools?".to_string()
    } else {
        format!("/admin/mcp/tools?{qs}&")
    };
    let prev_url = (page > 0).then(|| format!("{prefix}page={}", page - 1));
    let next_url = (page + 1 < total_pages).then(|| format!("{prefix}page={}", page + 1));
    json!({
        "current_page": page + 1,
        "total_pages": total_pages,
        "has_prev": prev_url.is_some(),
        "has_next": next_url.is_some(),
        "prev_url": prev_url,
        "next_url": next_url,
    })
}
