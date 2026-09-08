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

mod view;

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use sqlx::PgPool;
use systemprompt::loader::ServicesBootstrap;

use crate::error::AdminHtmlResult;
use crate::handlers::shared;
use crate::handlers::ssr::ssr_helpers::render_typed_page;
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{GatewayConfigView, GatewayRouteView, MarketplaceContext, UserContext};

use view::{
    GatewayKpiView, GatewayPageData, GatewayRouteRow, ProbeAccountView, ProviderOptionView,
    ResolvedOnlyRow,
};

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
fn yaml_summary(value: Option<&serde_yaml::Value>) -> String {
    let Some(value) = value else {
        return String::new();
    };
    serde_yaml::to_string(value)
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && *l != "---")
        .collect::<Vec<_>>()
        .join(", ")
}

struct Surfaces {
    providers: Vec<ProviderOptionView>,
}

impl Surfaces {
    fn surface_of(&self, provider: &str) -> (String, &'static str) {
        match self.providers.iter().find(|p| p.name == provider) {
            None => ("unknown".to_owned(), "err"),
            Some(p) if p.advertised => (p.surface.clone(), "ok"),
            Some(p) => (p.surface.clone(), "warn"),
        }
    }
}

fn load_surfaces() -> Surfaces {
    let providers = ServicesBootstrap::get().map_or_else(
        |_| Vec::new(),
        |services| {
            services
                .providers
                .providers
                .iter()
                .map(|p| ProviderOptionView {
                    name: p.name.as_str().to_owned(),
                    surface: p.surface.as_tag().to_owned(),
                    model_count: p.models.len(),
                    advertised: p.surface.is_advertised(),
                })
                .collect()
        },
    );
    Surfaces { providers }
}

fn route_rows(
    config: &GatewayConfigView,
    surfaces: &Surfaces,
    grants: &std::collections::HashMap<String, i64>,
    dispatchable: &[String],
) -> Vec<GatewayRouteRow> {
    let last = config.routes.len().saturating_sub(1);
    config
        .routes
        .iter()
        .enumerate()
        .map(|(index, route)| {
            let (surface, surface_tone) = surfaces.surface_of(&route.provider);
            GatewayRouteRow {
                index,
                surface,
                surface_tone,
                upstream_model: route.upstream_model.clone().unwrap_or_default(),
                requires: yaml_summary(route.requires.as_ref()),
                when: yaml_summary(route.when.as_ref()),
                has_pricing: route.pricing.is_some(),
                header_count: route.extra_headers.len(),
                grants: grants.get(&route.id).copied().unwrap_or(0),
                dispatchable: dispatchable.contains(&route.id),
                is_first: index == 0,
                is_last: index == last,
                matrix_url: format!(
                    "/admin/access-control?entity_type=gateway_route&entity_id={}",
                    route.id
                ),
                id: route.id.clone(),
                model_pattern: route.model_pattern.clone(),
                provider: route.provider.clone(),
            }
        })
        .collect()
}

fn resolved_only(
    declared: &[GatewayRouteRow],
    resolved: &[GatewayRouteView],
) -> Vec<ResolvedOnlyRow> {
    resolved
        .iter()
        .filter(|r| !declared.iter().any(|d| d.id == r.id))
        .map(|r| ResolvedOnlyRow {
            matrix_url: format!(
                "/admin/access-control?entity_type=gateway_route&entity_id={}",
                r.id
            ),
            id: r.id.clone(),
            model_pattern: r.model_pattern.clone(),
            provider: r.provider.clone(),
            upstream_model: r.upstream_model.clone().unwrap_or_default(),
        })
        .collect()
}

fn kpis(
    config: &GatewayConfigView,
    rows: &[GatewayRouteRow],
    resolved_extra: usize,
    surfaces: &Surfaces,
) -> Vec<GatewayKpiView> {
    let backend = rows.iter().filter(|r| r.surface == "backend").count();
    let unknown = rows.iter().filter(|r| r.surface == "unknown").count();
    let grants: i64 = rows.iter().map(|r| r.grants).sum();
    let governed = rows.iter().filter(|r| !r.requires.is_empty()).count();
    vec![
        GatewayKpiView {
            label: "Gateway",
            value: if config.enabled { "On" } else { "Off" }.to_owned(),
            sub: format!("{} on {}", config.auth_scheme, config.inference_path_prefix),
            tone: if config.enabled { "ok" } else { "warn" },
        },
        GatewayKpiView {
            label: "Routes declared",
            value: rows.len().to_string(),
            sub: format!("{resolved_extra} resolved but not in the file"),
            tone: "",
        },
        GatewayKpiView {
            label: "Providers",
            value: surfaces.providers.len().to_string(),
            sub: format!("{backend} routes on a backend-only provider"),
            tone: "",
        },
        GatewayKpiView {
            label: "Unknown provider",
            value: unknown.to_string(),
            sub: "routes naming a provider the registry lacks".to_owned(),
            tone: if unknown > 0 { "err" } else { "ok" },
        },
        GatewayKpiView {
            label: "Governed routes",
            value: governed.to_string(),
            sub: "carry a requires: classification".to_owned(),
            tone: "",
        },
        GatewayKpiView {
            label: "Route grants",
            value: grants.to_string(),
            sub: "access-control rules on routes".to_owned(),
            tone: "",
        },
    ]
}

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
        match repositories::config::gateway::get_gateway_config_from_file(&gateway_path) {
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
        source_path: config.config_path.clone(),
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
