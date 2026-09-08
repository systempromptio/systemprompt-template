//! Read-only catalog admin pages for the three installable entity families:
//!
//! - `/admin/plugins` — plugins (collections) from
//!   `services/plugins/*/config.yaml`, each referencing skills, MCP servers,
//!   agents, and hooks.
//! - `/admin/skills` — skills from `services/skills/*`.
//! - `/admin/mcp` — MCP servers from `services/mcp/*`, whose pages live in
//!   [`mcp`] because they also read the runtime tables.
//!
//! Each family has a list page and a detail page. Detail pages surface the
//! plugin ↔ member relationship in both directions. The plugin and skill pages
//! are strictly read-only: operators edit `services/*.yaml` and restart.

mod data;
mod entries;
pub(crate) mod marketplaces;
pub(crate) mod mcp;
pub(crate) mod sorting;
mod view;
mod view_models;
mod visibility;

use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::response::Response;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::shared;
use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{ENTITY_PLUGIN, ENTITY_SKILL, MarketplaceContext, UserContext};

use super::ssr::ssr_helpers::render_typed_page;
use crate::handlers::ssr::types::BreadcrumbView;
use entries::entry_columns;
use sorting::{apply_direction, direction, matches, preserved_search, sort_headers};
use view::assignment_counts_by_type;
use view_models::VisibilityInput;

#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct CatalogListQuery {
    pub sort: Option<String>,
    pub dir: Option<String>,
    // Why: the filter is a server round trip rather than a class toggle, so the
    // row count, the empty state and the URL all agree — a client-side hide
    // leaves the filtered rows in the DOM and the count above them wrong.
    pub q: Option<String>,
    // Why: which view a listing that has more than one is showing. Only the
    // marketplaces listing reads it today; the field is on the shared query so
    // the three listings keep one parameter set between them.
    pub tab: Option<String>,
}

// Why: two cheap reads the badge needs on every catalog page. Both are
// best-effort: a badge that cannot be computed renders as restricted, which is
// the safe reading, rather than failing the page.
async fn visibility_inputs(
    pool: &PgPool,
    services_path: &std::path::Path,
) -> (
    Vec<repositories::marketplace::manifests::MarketplaceConfigSummary>,
    Vec<crate::types::access_control::AccessControlRule>,
) {
    let manifests = repositories::marketplace::manifests::list_marketplace_configs(services_path)
        .unwrap_or_default();
    let rules = repositories::users::access_control::list_all_rules(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "catalog: rule listing failed"))
        .unwrap_or_default();
    (manifests, rules)
}

pub(crate) async fn plugins_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    axum::extract::Query(query): axum::extract::Query<CatalogListQuery>,
) -> AdminHtmlResult<Response> {
    admin_only(&user_ctx)?;
    let path = shared::get_services_path()?;

    let catalog = data::load_catalog(&path, &user_ctx.roles);
    let counts = assignment_counts_by_type(&pool, ENTITY_PLUGIN).await;
    let (manifests, rules) = visibility_inputs(&pool, &path).await;
    let mut plugins = view_models::plugin_rows(
        catalog,
        &counts,
        &VisibilityInput {
            manifests: &manifests,
            rules: &rules,
        },
    );
    let search = query.q.unwrap_or_default();
    plugins.retain(|p| matches(&[&p.id, &p.name, &p.description, &p.category], &search));
    let sort_key = query.sort.unwrap_or_else(|| "name".to_owned());
    let dir = direction(query.dir.as_deref());
    match sort_key.as_str() {
        "status" => apply_direction(&mut plugins, dir, |a, b| a.enabled.cmp(&b.enabled)),
        "members" => apply_direction(&mut plugins, dir, |a, b| {
            a.skills_count.cmp(&b.skills_count)
        }),
        "visibility" => apply_direction(&mut plugins, dir, |a, b| {
            a.visibility.is_public.cmp(&b.visibility.is_public)
        }),
        "grants" => apply_direction(&mut plugins, dir, |a, b| {
            a.assignment_count.cmp(&b.assignment_count)
        }),
        _ => apply_direction(&mut plugins, dir, |a, b| b.name.cmp(&a.name)),
    }

    let page = view::PluginsPageData {
        page: "plugins",
        title: "Plugins",
        subtitle: "Collections. A plugin bundles skills, MCP servers, agents and hooks, and a marketplace ships plugins.",
        breadcrumbs: vec![BreadcrumbView::current("Plugins")],
        kpis: entries::plugin_kpis(&plugins),
        sort_headers: sort_headers(
            "/admin/plugins",
            &entry_columns(
                "Members",
                "Skills, MCP servers and agents this plugin carries",
            ),
            &sort_key,
            dir,
            &preserved_search(&search),
        ),
        plugins_count: plugins.len(),
        plugins,
        access_control_url: "/admin/access-control?entity_type=plugin",
        search,
    };
    Ok(render_typed_page(
        &engine,
        "catalog-plugins",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}

pub(crate) async fn plugin_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
) -> AdminHtmlResult<Response> {
    admin_only(&user_ctx)?;
    let path = shared::get_services_path()?;

    let catalog = data::load_catalog(&path, &user_ctx.roles);
    let counts = assignment_counts_by_type(&pool, ENTITY_PLUGIN).await;
    let assignment_count = counts.get(&plugin_id).copied().unwrap_or(0);
    let page = view_models::plugin_detail(&catalog, &plugin_id, assignment_count)
        .ok_or_else(|| AdminError::NotFound("No such plugin.".to_owned()))?;
    Ok(render_typed_page(
        &engine,
        "catalog-plugin-detail",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}

pub(crate) async fn skills_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    axum::extract::Query(query): axum::extract::Query<CatalogListQuery>,
) -> AdminHtmlResult<Response> {
    admin_only(&user_ctx)?;
    let path = shared::get_services_path()?;

    let catalog = data::load_catalog(&path, &user_ctx.roles);
    let counts = assignment_counts_by_type(&pool, ENTITY_SKILL).await;
    let (manifests, rules) = visibility_inputs(&pool, &path).await;
    let mut skills = view_models::skill_rows(
        &catalog,
        &counts,
        &VisibilityInput {
            manifests: &manifests,
            rules: &rules,
        },
    );
    let search = query.q.unwrap_or_default();
    skills.retain(|s| matches(&[&s.id, &s.name, &s.description], &search));
    let sort_key = query.sort.unwrap_or_else(|| "name".to_owned());
    let dir = direction(query.dir.as_deref());
    match sort_key.as_str() {
        "status" => apply_direction(&mut skills, dir, |a, b| a.enabled.cmp(&b.enabled)),
        "members" => apply_direction(&mut skills, dir, |a, b| a.plugin_count.cmp(&b.plugin_count)),
        "visibility" => apply_direction(&mut skills, dir, |a, b| {
            a.visibility.is_public.cmp(&b.visibility.is_public)
        }),
        "grants" => apply_direction(&mut skills, dir, |a, b| {
            a.assignment_count.cmp(&b.assignment_count)
        }),
        _ => apply_direction(&mut skills, dir, |a, b| b.name.cmp(&a.name)),
    }

    let page = view::SkillsPageData {
        page: "skills",
        title: "Skills",
        subtitle: "The instruction sets people invoke. A skill reaches someone through the plugins that include it.",
        breadcrumbs: vec![BreadcrumbView::current("Skills")],
        kpis: entries::skill_kpis(&skills, manifests.len()),
        sort_headers: sort_headers(
            "/admin/skills",
            &entry_columns("Plugins", "How many plugins include this skill"),
            &sort_key,
            dir,
            &preserved_search(&search),
        ),
        skills_count: skills.len(),
        skills,
        access_control_url: "/admin/access-control?entity_type=skill",
        search,
    };
    Ok(render_typed_page(
        &engine,
        "catalog-skills",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}

pub(crate) async fn skill_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(skill_id): Path<String>,
) -> AdminHtmlResult<Response> {
    admin_only(&user_ctx)?;
    let path = shared::get_services_path()?;

    let catalog = data::load_catalog(&path, &user_ctx.roles);
    let counts = assignment_counts_by_type(&pool, ENTITY_SKILL).await;
    let assignment_count = counts.get(&skill_id).copied().unwrap_or(0);
    let skill = systemprompt::identifiers::SkillId::new(&skill_id);
    let page = view_models::skill_detail(&catalog, &skill, assignment_count)
        .ok_or_else(|| AdminError::NotFound("No such skill.".to_owned()))?;
    Ok(render_typed_page(
        &engine,
        "catalog-skill-detail",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}

fn admin_only(user_ctx: &UserContext) -> AdminHtmlResult<()> {
    if user_ctx.is_console {
        return Ok(());
    }
    Err(AdminError::Forbidden("Admin access required.".to_owned()).into())
}
