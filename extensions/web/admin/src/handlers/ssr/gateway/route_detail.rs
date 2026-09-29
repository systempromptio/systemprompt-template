//! `/admin/gateway/routes/{route_id}` — one model route: where it dispatches,
//! and the shared "Who gets this" panel for the `gateway_route` entity.
//!
//! `rules.yaml` declares gateway routes only through the glob
//! `gateway_route/*`, which the declared set expands to every dispatchable
//! route; a rule written here names this route alone and is a console rule.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::Response;
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::RouteId;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::entity_panel::{EntityAccessView, PanelRequest, build_entity_panel};
use crate::handlers::ssr::page::Page;
use crate::handlers::ssr::ssr_helpers::render_typed_page;
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::config::gateway::get_route_labels_from_services;

use super::ENTITY_GATEWAY_ROUTE;

pub(super) const ROUTES_URL: &str = "/admin/gateway/routes";

const GLOB_NOTE: &str = "Code declares gateway routes only as gateway_route/* in rules.yaml: those rules apply to every route and are listed here as they were applied to this one. A rule added here names this route alone and lives in the console until exported.";

#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct RouteQuery {
    why: Option<String>,
}

#[derive(Debug, Serialize)]
struct RouteFactView {
    label: &'static str,
    value: String,
}

#[derive(Debug, Serialize)]
struct GatewayRoutePageData {
    page: &'static str,
    title: String,
    subtitle: String,
    breadcrumbs: Vec<BreadcrumbView>,
    route_id: RouteId,
    facts: Vec<RouteFactView>,
    access: EntityAccessView,
}

pub(crate) async fn gateway_route_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(route_id): Path<RouteId>,
    Query(query): Query<RouteQuery>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let labels = get_route_labels_from_services()
        .inspect_err(|e| tracing::warn!(error = %e, "gateway route: labels unavailable"))
        .unwrap_or_default();
    let route = labels
        .routes
        .iter()
        .find(|r| r.id == route_id.as_str())
        .ok_or_else(|| AdminError::NotFound("No such gateway route.".to_owned()))?;
    let page_url = format!("{ROUTES_URL}/{}", urlencoding::encode(route_id.as_str()));
    let access = build_entity_panel(
        &pool,
        PanelRequest {
            entity_type: ENTITY_GATEWAY_ROUTE,
            entity_id: route_id.as_str(),
            page_url: &page_url,
            why: query.why.as_deref(),
            can_write: shell.user.is_admin,
            note: Some(GLOB_NOTE),
        },
    )
    .await;
    let fact = |label: &'static str, value: String| RouteFactView { label, value };
    let mut facts = vec![
        fact("Route id", route.id.clone()),
        fact("Model pattern", route.model_pattern.clone()),
        fact("Provider", route.provider_label.clone()),
        fact("Endpoint", route.endpoint.clone()),
    ];
    if let Some(upstream) = &route.upstream {
        facts.push(fact("Upstream model", upstream.clone()));
    }
    if let Some(fallback) = &route.fallback {
        facts.push(fact("Fallback", fallback.clone()));
    }
    let page = GatewayRoutePageData {
        page: "gateway",
        title: route.label.clone(),
        subtitle: route.description.clone().unwrap_or_default(),
        breadcrumbs: vec![
            BreadcrumbView::link("Gateway", "/admin/gateway?tab=routes"),
            BreadcrumbView::current(route.label.clone()),
        ],
        route_id,
        facts,
        access,
    };
    Ok(render_typed_page(
        &shell.engine,
        "gateway-route",
        &page,
        &shell.user,
        &shell.marketplace,
    ))
}
