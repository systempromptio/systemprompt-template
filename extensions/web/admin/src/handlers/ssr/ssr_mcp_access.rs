use std::sync::Arc;

use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

use super::ACCESS_DENIED_HTML;

/// SSR page for `/admin/mcp/access` — OAuth events, access grants and
/// per-user governance decisions captured by the MCP access tracking demo.
pub async fn mcp_access_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let (access_events_res, governance_res, counts_res) = tokio::join!(
        repositories::dashboard_queries::fetch_mcp_access_events(&pool),
        repositories::governance::list_governance_decisions(&pool, None),
        repositories::governance::fetch_governance_counts(&pool),
    );

    let access_events = access_events_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch MCP access events");
        vec![]
    });
    let governance = governance_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list governance decisions");
        vec![]
    });
    let counts = counts_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch governance counts");
        repositories::governance::GovernanceCounts {
            total: 0,
            allowed: 0,
            denied: 0,
            secret_breaches: 0,
        }
    });

    // JSON: template context for Handlebars rendering
    let access_events_json: Vec<serde_json::Value> = access_events
        .iter()
        .map(|e| {
            json!({
                "server_name": e.server_name,
                "action": e.action,
                "description": e.description,
                "created_at": e.created_at,
            })
        })
        .collect();

    let governance_json: Vec<serde_json::Value> = governance
        .iter()
        .take(50)
        .map(|g| {
            json!({
                "id": g.id,
                "user_id": g.user_id,
                "tool_name": g.tool_name,
                "agent_id": g.agent_id,
                "agent_scope": g.agent_scope,
                "decision": g.decision,
                "is_denied": g.decision == "deny",
                "policy": g.policy,
                "reason": g.reason,
                "created_at": g.created_at,
            })
        })
        .collect();

    let data = json!({
        "page": "mcp-access",
        "title": "MCP Access Tracking",
        "cli_command": "systemprompt infra db query \"SELECT * FROM user_activity WHERE category='mcp_access'\"",
        "demo_script": "demo/mcp/02-mcp-access-tracking.sh",
        "page_stats": [
            {"key": "allowed", "value": counts.allowed, "label": "Allowed"},
            {"key": "denied", "value": counts.denied, "label": "Denied"},
            {"key": "secrets", "value": counts.secret_breaches, "label": "Secret breaches"},
            {"key": "events", "value": access_events_json.len(), "label": "Access events"},
        ],
        "access_events": access_events_json,
        "has_access_events": !access_events_json.is_empty(),
        "governance": governance_json,
        "has_governance": !governance_json.is_empty(),
    });
    super::render_page(&engine, "mcp-access", &data, &user_ctx, &mkt_ctx)
}
