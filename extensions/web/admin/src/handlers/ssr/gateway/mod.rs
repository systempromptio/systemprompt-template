//! `/admin/gateway` — the model-routing table and the settings above it.
//!
//! This is the one editing surface in the platform section: everything else on
//! these pages is read from YAML and changed by an operator with a text editor.
//! The gateway is different because route order is policy — the first pattern
//! that matches wins — and reordering a file by hand while traffic is flowing
//! is how a rewrite ends up pointing at the wrong provider.
//!
//! Every mutation goes through the existing JSON API (`PATCH /gateway`,
//! `POST|PATCH|DELETE /gateway/routes`). Routes are read from the
//! `gateway_routes` table, which every write lands in before the file's
//! `routes:` sequence is regenerated from it; core boots from that file, so
//! a saved route is dispatched at the next restart and the page says so.
//! `pricing`/`when`/`requires` blocks the form does not render are carried
//! through untouched. The page never writes YAML itself.

mod models;
mod route_detail;
mod rows;
mod view;

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use sqlx::PgPool;

use crate::error::AdminHtmlResult;
use crate::handlers::ssr::pickable_users::list_pickable_users;
use crate::handlers::ssr::ssr_helpers::render_typed_page;
use crate::handlers::ssr::types::{BreadcrumbView, TabLinkView};
use crate::repositories;
use crate::repositories::config::gateway::{RouteLabels, get_route_labels_from_services};
use crate::templates::AdminTemplateEngine;
use crate::types::{GatewayConfigView, MarketplaceContext, UserContext};

pub(crate) use route_detail::gateway_route_page;
use rows::{kpis, load_surfaces, resolved_only, route_rows};
use view::GatewayPageData;

const ENTITY_GATEWAY_ROUTE: &str = "gateway_route";

const NAMES_NOTE: &str = "A route's name and description are written to services/ai/gateway.yaml with the route; a provider's display name lives in services/ai/providers.yaml. The rest of the console reads them from the booted services, so a name edited here shows elsewhere after the same restart.";

#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct GatewayQuery {
    tab: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GatewayTab {
    Overview,
    Providers,
    Routes,
    Settings,
}

impl GatewayTab {
    // Why: `resolved` was its own tab; the resolved-only set now sits under
    // Routes, and old links keep landing on it.
    fn parse(tab: Option<&str>) -> Self {
        match tab {
            Some("providers") => Self::Providers,
            Some("routes" | "resolved") => Self::Routes,
            Some("settings") => Self::Settings,
            _ => Self::Overview,
        }
    }
}

// Why: five views, each a URL. Overview is the summary an operator scans —
// dispatch order and provider health as two tables; Providers opens each
// upstream into its models and routes; Routes is the ordered table the
// dispatcher reads, where order is edited, with the resolved-only set below
// it; Settings holds the switches and the per-account probe; Policies lives
// on its own page and is linked so the gateway reads as one thing.
fn view_tabs(active: GatewayTab, routes: usize, providers: usize) -> Vec<TabLinkView> {
    let link = |slug, label, href: &str, tab: Option<GatewayTab>, count| TabLinkView {
        slug,
        label,
        href: href.to_owned(),
        is_active: tab == Some(active),
        count,
    };
    vec![
        link(
            "overview",
            "Overview",
            "/admin/gateway",
            Some(GatewayTab::Overview),
            None,
        ),
        link(
            "providers",
            "Providers",
            "/admin/gateway?tab=providers",
            Some(GatewayTab::Providers),
            Some(i64::try_from(providers).unwrap_or(i64::MAX)),
        ),
        link(
            "routes",
            "Routes",
            "/admin/gateway?tab=routes",
            Some(GatewayTab::Routes),
            Some(i64::try_from(routes).unwrap_or(i64::MAX)),
        ),
        link(
            "settings",
            "Settings",
            "/admin/gateway?tab=settings",
            Some(GatewayTab::Settings),
            None,
        ),
        link(
            "policies",
            "Policies",
            "/admin/gateway/policies",
            None,
            None,
        ),
    ]
}

fn console_only(user_ctx: &UserContext) -> AdminHtmlResult<()> {
    if user_ctx.is_console {
        return Ok(());
    }
    Err(crate::error::AdminError::Forbidden("Admin access required.".to_owned()).into())
}

pub(crate) async fn gateway_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<GatewayQuery>,
) -> AdminHtmlResult<Response> {
    console_only(&user_ctx)?;

    let (config, load_error) = match super::super::gateway_config_view(&pool).await {
        Ok(config) => (config, String::new()),
        Err(e) => (GatewayConfigView::default(), e.to_string()),
    };
    let (resolved, catalog_error) =
        match repositories::config::gateway::dispatchable_routes_from_services() {
            Ok(routes) => (routes, String::new()),
            Err(e) => (Vec::new(), e.to_string()),
        };

    let labels: RouteLabels = get_route_labels_from_services()
        .inspect_err(|e| tracing::warn!(error = %e, "gateway: route labels unavailable"))
        .unwrap_or_default();
    let surfaces = load_surfaces(&labels);
    let grants = repositories::users::access_control::count_assignments_by_entity_type(
        &pool,
        ENTITY_GATEWAY_ROUTE,
    )
    .await
    .inspect_err(|e| tracing::warn!(error = %e, "gateway: route rule listing failed"))
    .unwrap_or_default();
    let dispatchable: Vec<String> = resolved.iter().map(|r| r.id.clone()).collect();

    let tab = GatewayTab::parse(query.tab.as_deref());
    let routes = route_rows(&config, &surfaces, &grants, &dispatchable, &labels);
    let extra = resolved_only(&routes, &resolved, &labels);
    let overview = matches!(tab, GatewayTab::Overview | GatewayTab::Providers);
    let provider_cards = if overview {
        models::provider_cards(&labels, &routes, &grants)
    } else {
        Vec::new()
    };
    let dispatch = if tab == GatewayTab::Overview {
        models::dispatch_rows(&labels, &routes, &grants)
    } else {
        Vec::new()
    };

    let routes_count = routes.len();
    let providers_count = surfaces.providers.len();
    let page = GatewayPageData {
        page: "gateway",
        title: "Gateway",
        subtitle: "Which providers this instance can reach, the models they serve, and which requested model goes where.",
        breadcrumbs: vec![BreadcrumbView::current("Gateway")],
        enabled: config.enabled,
        auth_scheme: config.auth_scheme.clone(),
        inference_path_prefix: config.inference_path_prefix.clone(),
        source_path: config.source_path.clone(),
        kpis: kpis(&config, &routes, extra.len(), &surfaces),
        routes_count,
        routes,
        resolved_only_count: extra.len(),
        resolved_only: extra,
        providers_count,
        providers: surfaces.providers,
        probe_users: list_pickable_users(&pool, None).await,
        provider_cards,
        dispatch_count: dispatch.len(),
        dispatch,
        load_error,
        catalog_error,
        runtime_note: repositories::sync::gateway_routes::RUNTIME_NOTE,
        sync_url: "/admin/sync?entity=gateway_route",
        names_note: NAMES_NOTE,
        tabs: view_tabs(tab, routes_count, providers_count),
        show_overview: tab == GatewayTab::Overview,
        show_providers: tab == GatewayTab::Providers,
        show_routes: tab == GatewayTab::Routes,
        show_settings: tab == GatewayTab::Settings,
    };
    Ok(render_typed_page(
        &engine, "gateway", &page, &user_ctx, &mkt_ctx,
    ))
}
