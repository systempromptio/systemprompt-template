use std::sync::Arc;

use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use serde_json::json;
use sqlx::PgPool;

use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

pub(crate) async fn governance_hooks_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let services_path = match super::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let configured_hooks = repositories::list_configured_hooks(&services_path, &user_ctx.roles)
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "list_configured_hooks failed");
            Vec::new()
        });

    let pretool_fired = repositories::governance_grp::hook_events::count_pretool_fired_24h(&pool)
        .await
        .unwrap_or(0);
    let posttool_fired = repositories::governance_grp::hook_events::count_posttool_fired_24h(&pool)
        .await
        .unwrap_or(0);
    let recent_events = repositories::governance_grp::hook_events::recent_hook_events(&pool, 50)
        .await
        .unwrap_or_default();

    let recent_events_view: Vec<serde_json::Value> = recent_events
        .into_iter()
        .map(|e| {
            json!({
                "kind": e.kind,
                "created_at": e.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                "plugin_id": e.plugin_id.unwrap_or_default(),
                "tool_name": e.tool_name.unwrap_or_default(),
                "user_id": e.user_id,
                "status": e.status.unwrap_or_default(),
            })
        })
        .collect();

    let configured_count = configured_hooks.len();
    let has_recent = !recent_events_view.is_empty();
    let data = json!({
        "page": "governance-hooks",
        "title": "Governance Hooks",
        "hero_title": "Governance Hooks",
        "hero_subtitle": "Hooks declared in plugin YAMLs and a snapshot of recent ingested events.",
        "configured": configured_count,
        "pretool_fired": pretool_fired,
        "posttool_fired": posttool_fired,
        "configured_hooks": configured_hooks,
        "has_configured": configured_count > 0,
        "recent_events": recent_events_view,
        "has_recent": has_recent,
    });

    super::render_page(&engine, "governance-hooks", &data, &user_ctx, &mkt_ctx)
}
