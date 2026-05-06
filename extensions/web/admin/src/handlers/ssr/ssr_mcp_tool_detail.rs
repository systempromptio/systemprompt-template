//! `/admin/mcp/tools/:tool_name` — single-tool drill page.

use std::sync::Arc;

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;

use crate::repositories::governance_grp::time_range::{parse_time_range, TimeRangeQuery};
use crate::repositories::mcp_grp::tool_calls::{
    fetch_tool_calls_paged, fetch_tool_deny_reasons, fetch_tool_detail_stats,
    fetch_tool_top_actors, ToolActorGroup, ToolCallFilter, ToolCallRow, ToolDenyReason,
    ToolDetailStats, ToolSortSpec, ToolTopActor,
};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

const HISTORY_LIMIT: i64 = 100;
const DENY_REASONS_LIMIT: i64 = 10;
const ACTOR_LIMIT: i64 = 10;

#[derive(Debug, Deserialize)]
pub struct ToolDetailQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub preset: Option<String>,
}

pub async fn mcp_tool_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(tool_name): Path<String>,
    Query(query): Query<ToolDetailQuery>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let range = parse_time_range(&TimeRangeQuery {
        from: query.from.clone(),
        to: query.to.clone(),
        preset: query.preset.clone(),
    });

    let filter = ToolCallFilter {
        tool_name: Some(tool_name.clone()),
        ..ToolCallFilter::default()
    };
    let sort = ToolSortSpec::default();

    let (history_res, stats_res, deny_reasons_res, top_users_res, top_agents_res) = tokio::join!(
        fetch_tool_calls_paged(&pool, &filter, range, sort, HISTORY_LIMIT, 0),
        fetch_tool_detail_stats(&pool, &tool_name, range),
        fetch_tool_deny_reasons(&pool, &tool_name, range, DENY_REASONS_LIMIT),
        fetch_tool_top_actors(&pool, &tool_name, range, ToolActorGroup::User, ACTOR_LIMIT),
        fetch_tool_top_actors(&pool, &tool_name, range, ToolActorGroup::Agent, ACTOR_LIMIT),
    );

    let (history, total_count) = history_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_tool_calls_paged (detail) failed");
        (Vec::new(), 0)
    });
    let stats = stats_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_tool_detail_stats failed");
        ToolDetailStats {
            tool_name: tool_name.clone(),
            ..ToolDetailStats::default()
        }
    });
    let deny_reasons = deny_reasons_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_tool_deny_reasons failed");
        Vec::new()
    });
    let top_users = top_users_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_tool_top_actors (users) failed");
        Vec::new()
    });
    let top_agents = top_agents_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_tool_top_actors (agents) failed");
        Vec::new()
    });

    let data = json!({
        "page": "mcp-tools",
        "title": format!("{tool_name} — Tool"),
        "tool_name": tool_name,
        "back_url": "/admin/mcp/tools",
        "audit_url": format!(
            "/admin/governance/decisions?q={}",
            urlencoding::encode(&tool_name),
        ),
        "time_range": {
            "preset": query.preset.clone().unwrap_or_else(|| "24h".to_string()),
            "from": range.from.to_rfc3339(),
            "to": range.to.to_rfc3339(),
            "base_url": format!("/admin/mcp/tools/{}", urlencoding::encode(&stats.tool_name)),
            "query": "",
        },
        "stats": stats_to_json(&stats),
        "deny_reasons": deny_reasons.iter().map(deny_reason_to_json).collect::<Vec<_>>(),
        "has_deny_reasons": !deny_reasons.is_empty(),
        "top_users": top_users.iter().map(top_actor_to_json).collect::<Vec<_>>(),
        "top_agents": top_agents.iter().map(top_actor_to_json).collect::<Vec<_>>(),
        "history": history.iter().map(history_to_json).collect::<Vec<_>>(),
        "total_count": total_count,
        "has_history": !history.is_empty(),
    });
    super::render_page(&engine, "mcp-tool-detail", &data, &user_ctx, &mkt_ctx)
}

fn stats_to_json(s: &ToolDetailStats) -> serde_json::Value {
    let success_pct = if s.total_calls > 0 {
        (s.allow_count as f64 / s.total_calls as f64) * 100.0
    } else {
        0.0
    };
    json!({
        "total_calls": s.total_calls,
        "allow_count": s.allow_count,
        "deny_count": s.deny_count,
        "distinct_users": s.distinct_users,
        "distinct_agents": s.distinct_agents,
        "total_bytes_in": s.total_bytes_in,
        "total_bytes_out": s.total_bytes_out,
        "success_pct": format!("{success_pct:.1}"),
    })
}

fn deny_reason_to_json(r: &ToolDenyReason) -> serde_json::Value {
    json!({
        "reason": r.reason,
        "policy": r.policy,
        "count": r.count,
    })
}

fn top_actor_to_json(r: &ToolTopActor) -> serde_json::Value {
    json!({
        "identity_id": r.identity_id,
        "label": r.label,
        "deny_count": r.deny_count,
        "total_count": r.total_count,
    })
}

fn history_to_json(r: &ToolCallRow) -> serde_json::Value {
    json!({
        "id": r.id,
        "trace_id": r.trace_id,
        "session_id": r.session_id,
        "user_id": r.user_id,
        "agent_id": r.agent_id,
        "agent_scope": r.agent_scope,
        "decision": r.decision,
        "is_denied": r.decision.as_deref() == Some("deny"),
        "is_allowed": r.decision.as_deref() == Some("allow"),
        "policy": r.policy,
        "reason": r.reason,
        "content_input_bytes": r.content_input_bytes,
        "content_output_bytes": r.content_output_bytes,
        "latency_ms": r.ar_latency_ms,
        "metadata": r.metadata,
        "metadata_present": !r.metadata.is_null()
            && r.metadata.as_object().is_some_and(|m| !m.is_empty()),
        "created_at": r.created_at.to_rfc3339(),
        "created_at_local": r
            .created_at
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
    })
}
