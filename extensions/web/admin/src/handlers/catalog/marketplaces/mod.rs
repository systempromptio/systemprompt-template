//! `/admin/marketplaces` — the marketplace list with its audience
//! matrix, and the per-marketplace detail page.
//!
//! A marketplace is the unit entitlement is granted on, so these two pages
//! answer a question the plugin and skill pages cannot: not "what is in the
//! catalog" but "who reaches it". The manifests live in
//! `services/marketplaces/*/config.yaml`; who reaches one is read and edited
//! on its own detail page, in the shared "Who gets this" panel.

mod cards;
mod data;
mod kpis;
mod view;

use std::sync::Arc;
use systemprompt::identifiers::MarketplaceId;

use axum::extract::{Extension, Path, Query, State};
use axum::response::Response;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::shared;
use crate::handlers::ssr::entity_panel::{PanelRequest, build_entity_panel};
use crate::handlers::ssr::page::Page;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, Role, UserContext};

use self::kpis::list_kpis;
use self::view::{
    MarketplaceCardView, MarketplaceDetailData, MarketplacesPageData, MemberLinkView,
};
use super::super::ssr::ssr_helpers::render_typed_page;
use super::view::{PanelQuery, mcp_url, plugin_url, skill_url};
use crate::handlers::ssr::types::BreadcrumbView;

// Why: the two views the listing offers. Links, not script, so a matrix an
// operator wants to send someone is a URL.
fn audience_tabs(show_audience: bool) -> Vec<crate::handlers::ssr::types::TabLinkView> {
    vec![
        crate::handlers::ssr::types::TabLinkView {
            slug: "list",
            label: "Marketplaces",
            href: "/admin/marketplaces".to_owned(),
            is_active: !show_audience,
            count: None,
        },
        crate::handlers::ssr::types::TabLinkView {
            slug: "audience",
            label: "Audience matrix",
            href: "/admin/marketplaces?tab=audience".to_owned(),
            is_active: show_audience,
            count: None,
        },
    ]
}

fn known_roles() -> Vec<String> {
    Role::ALL.iter().map(|r| r.as_str().to_owned()).collect()
}

fn console_only(user_ctx: &UserContext) -> AdminHtmlResult<()> {
    if user_ctx.is_console {
        return Ok(());
    }
    Err(AdminError::Forbidden("Admin access required.".to_owned()).into())
}

pub(crate) async fn marketplaces_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<super::CatalogListQuery>,
) -> AdminHtmlResult<Response> {
    console_only(&user_ctx)?;
    let path = shared::get_services_path()?;
    let manifests = data::load_manifests(&pool, &path).await;
    let audience = data::audience_matrix(&pool, &manifests, &known_roles()).await;
    let grants = data::group_grants(&pool).await;
    let plugin_catalog = crate::repositories::marketplace::plugins::list_plugin_catalog(&path)
        .inspect_err(|e| tracing::warn!(error = %e, "marketplaces: plugin catalog failed"))
        .unwrap_or_default();
    let versions = cards::current_hashes(&pool).await;
    let marketplaces = cards::card_views(&cards::CardInputs {
        manifests: &manifests,
        audience: &audience,
        grants: &grants,
        plugin_catalog: &plugin_catalog,
        versions: &versions,
    });

    // Why: two views rather than two tables stacked. The estate-wide matrix is
    // a different question from the listing — "who reaches what across all of
    // them" against "what is each one" — and stacking it pushed the listing's
    // own rows off the fold on a 900px screen.
    let show_audience = query.tab.as_deref() == Some("audience");
    let search = query.q.unwrap_or_default();
    let kpis = list_kpis(&marketplaces, &audience);
    let marketplaces: Vec<MarketplaceCardView> = marketplaces
        .into_iter()
        .filter(|m| super::sorting::matches(&[m.id.as_str(), &m.name, &m.description], &search))
        .collect();

    let page = MarketplacesPageData {
        page: "marketplaces",
        title: "Marketplaces",
        subtitle: "The unit entitlement is granted on. Each one names plugins and MCP servers, and each group holds a set of them.",
        breadcrumbs: vec![BreadcrumbView::current("Marketplaces")],
        // Why: the tiles count every marketplace, not the filtered view — a
        // total that moved with the search box would mean two different things.
        kpis,
        marketplaces_count: marketplaces.len(),
        marketplaces,
        audience,
        access_control_url: "/admin/access-control?entity_type=marketplace",
        tabs: audience_tabs(show_audience),
        show_audience,
        search,
    };
    Ok(render_typed_page(
        &engine,
        "catalog-marketplaces",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}

fn member_links(ids: &[String], url: fn(&str) -> String) -> Vec<MemberLinkView> {
    ids.iter()
        .map(|id| MemberLinkView {
            id: id.clone(),
            name: id.clone(),
            url: url(id),
        })
        .collect()
}

// Why: a marketplace names plugins, and skills follow the plugins that ship
// them — there is no marketplace-level skill list to read, so the skills shown
// here are derived from the member plugins rather than declared. The catalog
// is passed in because the list page needs this once per marketplace and
// re-reading the plugin tree each time made one page render walk it four times.
pub(super) fn skills_of(
    catalog: &[crate::types::PluginDetail],
    plugin_ids: &[String],
) -> Vec<MemberLinkView> {
    let mut out: Vec<MemberLinkView> = Vec::new();
    for plugin in catalog
        .iter()
        .filter(|p| plugin_ids.iter().any(|id| id == p.id.as_str()))
    {
        for skill in &plugin.skills {
            let id = skill.as_str().to_owned();
            if out.iter().any(|s| s.id == id) {
                continue;
            }
            out.push(MemberLinkView {
                url: skill_url(&id),
                name: id.clone(),
                id,
            });
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

pub(crate) async fn marketplace_detail_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(marketplace_id): Path<MarketplaceId>,
    Query(query): Query<PanelQuery>,
) -> AdminHtmlResult<Response> {
    let Page {
        engine,
        user: user_ctx,
        marketplace: mkt_ctx,
    } = shell;
    console_only(&user_ctx)?;
    let path = shared::get_services_path()?;
    let manifests = data::load_manifests(&pool, &path).await;
    let manifest = manifests
        .iter()
        .find(|m| m.id == marketplace_id)
        .cloned()
        .ok_or_else(|| AdminError::NotFound("No such marketplace.".to_owned()))?;

    let page_url = view::marketplace_url(marketplace_id.as_str());
    let access = build_entity_panel(
        &pool,
        PanelRequest {
            entity_type: data::MARKETPLACE_ENTITY,
            entity_id: marketplace_id.as_str(),
            page_url: &page_url,
            why: query.why.as_deref(),
            can_write: user_ctx.is_admin,
            note: None,
        },
    )
    .await;

    let plugins = member_links(&manifest.plugins, plugin_url);
    let mcp_servers = member_links(&manifest.mcp_servers, mcp_url);
    let plugin_catalog = crate::repositories::marketplace::plugins::list_plugin_catalog(&path)
        .inspect_err(|e| tracing::warn!(error = %e, "marketplaces: plugin catalog failed"))
        .unwrap_or_default();
    let skills = skills_of(&plugin_catalog, &manifest.plugins);

    let page = MarketplaceDetailData {
        page: "marketplace-detail",
        title: manifest.name.clone(),
        breadcrumbs: vec![
            BreadcrumbView::link("Marketplaces", "/admin/marketplaces"),
            BreadcrumbView::current(manifest.name.clone()),
        ],
        id: manifest.id,
        name: manifest.name,
        description: manifest.description,
        version: manifest.version,
        enabled: manifest.enabled,
        visibility: manifest.visibility,
        source_path: manifest.source_path,
        plugins_count: plugins.len(),
        skills_count: skills.len(),
        mcp_count: mcp_servers.len(),
        plugins,
        skills,
        mcp_servers,
        access,
    };
    Ok(render_typed_page(
        &engine,
        "catalog-marketplace-detail",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}
