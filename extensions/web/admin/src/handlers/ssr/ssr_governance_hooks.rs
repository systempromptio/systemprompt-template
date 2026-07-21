//! SSR page showing recent hook events and their governance outcome.

use crate::error::AdminError;
use std::sync::Arc;

use axum::extract::{Extension, State};
use axum::response::Response;
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::error::AdminHtmlResult;
use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};


#[derive(Debug, Serialize)]
struct GovernanceHooksContext {
    page: &'static str,
    title: &'static str,
    hero_title: &'static str,
    hero_subtitle: &'static str,
    configured: usize,
    pretool_fired: i64,
    posttool_fired: i64,
    configured_hooks: Vec<crate::types::ConfiguredHook>,
    has_configured: bool,
    recent_events: Vec<RecentEventRow>,
    has_recent: bool,
}

#[derive(Debug, Serialize)]
struct RecentEventRow {
    kind: String,
    created_at: String,
    plugin_id: String,
    tool_name: String,
    user_id: UserId,
    status: String,
}

pub(crate) async fn governance_hooks_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let services_path = super::get_services_path()?;

    let configured_hooks =
        repositories::marketplace::hooks::list_configured_hooks(&services_path, &user_ctx.roles)
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "list_configured_hooks failed");
                Vec::new()
            });

    let pretool_fired = repositories::governance::hook_events::count_pretool_fired_24h(&pool)
        .await
        .unwrap_or(0);
    let posttool_fired = repositories::governance::hook_events::count_posttool_fired_24h(&pool)
        .await
        .unwrap_or(0);
    let recent_events = repositories::governance::hook_events::recent_hook_events(&pool, 50)
        .await
        .unwrap_or_default();

    let recent_events_view: Vec<RecentEventRow> = recent_events
        .into_iter()
        .map(|e| RecentEventRow {
            kind: e.kind,
            created_at: e.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            plugin_id: e
                .plugin_id
                .map(|p| p.as_str().to_owned())
                .unwrap_or_default(),
            tool_name: e.tool_name.unwrap_or_default(),
            user_id: e.user_id,
            status: e.status.unwrap_or_default(),
        })
        .collect();

    let configured_count = configured_hooks.len();
    let has_recent = !recent_events_view.is_empty();
    let ctx = GovernanceHooksContext {
        page: "governance-hooks",
        title: "Governance Hooks",
        hero_title: "Governance Hooks",
        hero_subtitle: "Hooks declared in plugin YAMLs and a snapshot of recent ingested events.",
        configured: configured_count,
        pretool_fired,
        posttool_fired,
        has_configured: configured_count > 0,
        configured_hooks,
        recent_events: recent_events_view,
        has_recent,
    };

    Ok(super::render_typed_page(
        &engine,
        "governance-hooks",
        &ctx,
        &user_ctx,
        &mkt_ctx,
    ))
}
