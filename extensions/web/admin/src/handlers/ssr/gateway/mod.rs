//! `/admin/gateway` — the model-routing table and the settings above it.
//!
//! This is the one editing surface in the platform section: everything else on
//! these pages is read from YAML and changed by an operator with a text editor.
//! The gateway is different because route order is policy — the first pattern
//! that matches wins — and reordering a file by hand while traffic is flowing
//! is how a rewrite ends up pointing at the wrong provider.
//!
//! Every mutation goes through the existing JSON API (`PATCH /gateway`,
//! `POST|PATCH|DELETE /gateway/routes`), which round-trips the file safely:
//! comments survive, and `pricing`/`when`/`requires` blocks the form does not
//! render are carried through untouched. The page never writes YAML itself.

mod rows;
mod view;

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use sqlx::PgPool;

use crate::error::AdminHtmlResult;
use crate::handlers::shared;
use crate::handlers::ssr::ssr_helpers::render_typed_page;
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{GatewayConfigView, MarketplaceContext, UserContext};

use rows::{kpis, load_surfaces, resolved_only, route_rows};
use view::{GatewayPageData, ProbeAccountView};

const PROBE_USER_LIMIT: usize = 200;
const ENTITY_GATEWAY_ROUTE: &str = "gateway_route";

#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct GatewayQuery {
    tab: Option<String>,
}

// Why: the declared table and the resolved one answer different questions, and
// stacking them put the routing table — the thing the page is for — below the
// fold on a 900px screen. Links rather than script, so a view is a URL.
fn view_tabs(show_resolved: bool) -> Vec<crate::handlers::ssr::types::TabLinkView> {
    vec![
        crate::handlers::ssr::types::TabLinkView {
            slug: "routes",
            label: "Routes",
            href: "/admin/gateway".to_owned(),
            is_active: !show_resolved,
            count: None,
        },
        crate::handlers::ssr::types::TabLinkView {
            slug: "resolved",
            label: "Resolved only",
            href: "/admin/gateway?tab=resolved".to_owned(),
            is_active: show_resolved,
            count: None,
        },
    ]
}

fn console_only(user_ctx: &UserContext) -> AdminHtmlResult<()> {
    if user_ctx.is_console {
        return Ok(());
    }
    Err(crate::error::AdminError::Forbidden("Admin access required.".to_owned()).into())
}

// Why: a YAML scalar or sequence rendered on one line. The editor does not
// parse these blocks — it round-trips them — so the page shows them as the
// operator wrote them rather than inventing a typed form for them.
async fn probe_users(pool: &PgPool) -> Vec<ProbeAccountView> {
    repositories::users::queries::list_users(pool, &repositories::scope::SubjectScope::All)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "gateway: user listing failed"))
        .unwrap_or_default()
        .into_iter()
        .take(PROBE_USER_LIMIT)
        .map(|u| {
            let id = u.user_id.as_str().to_owned();
            ProbeAccountView {
                label: u
                    .email
                    .as_ref()
                    .map(ToString::to_string)
                    .or(u.display_name)
                    .unwrap_or_else(|| id.clone()),
                id,
            }
        })
        .collect()
}

pub(crate) async fn gateway_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<GatewayQuery>,
) -> AdminHtmlResult<Response> {
    console_only(&user_ctx)?;
    let gateway_path = shared::get_gateway_file_path()?;

    let (config, load_error) =
        match repositories::config::gateway::get_gateway_config(&gateway_path) {
            Ok(config) => (config, String::new()),
            Err(e) => (GatewayConfigView::default(), e.to_string()),
        };
    let (resolved, catalog_error) =
        match repositories::config::gateway::dispatchable_routes_from_services() {
            Ok(routes) => (routes, String::new()),
            Err(e) => (Vec::new(), e.to_string()),
        };

    let surfaces = load_surfaces();
    let grants = repositories::users::access_control::count_assignments_by_entity_type(
        &pool,
        ENTITY_GATEWAY_ROUTE,
    )
    .await
    .inspect_err(|e| tracing::warn!(error = %e, "gateway: route rule listing failed"))
    .unwrap_or_default();
    let dispatchable: Vec<String> = resolved.iter().map(|r| r.id.clone()).collect();

    let show_resolved = query.tab.as_deref() == Some("resolved");
    let routes = route_rows(&config, &surfaces, &grants, &dispatchable);
    let extra = resolved_only(&routes, &resolved);

    let page = GatewayPageData {
        page: "gateway",
        title: "Gateway",
        subtitle: "Which model request goes to which provider, in the order the dispatcher tries them.",
        breadcrumbs: vec![BreadcrumbView::current("Gateway")],
        enabled: config.enabled,
        auth_scheme: config.auth_scheme.clone(),
        inference_path_prefix: config.inference_path_prefix.clone(),
        source_path: config.source_path.clone(),
        kpis: kpis(&config, &routes, extra.len(), &surfaces),
        routes_count: routes.len(),
        routes,
        resolved_only_count: extra.len(),
        resolved_only: extra,
        providers_count: surfaces.providers.len(),
        providers: surfaces.providers,
        probe_users: probe_users(&pool).await,
        load_error,
        catalog_error,
        tabs: view_tabs(show_resolved),
        show_resolved,
    };
    Ok(render_typed_page(
        &engine, "gateway", &page, &user_ctx, &mkt_ctx,
    ))
}
