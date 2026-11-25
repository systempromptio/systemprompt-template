//! `/admin/skills` and `/admin/skills/{id}` — the skill list and detail pages.

use crate::env::AdminEnv;
use crate::handlers::ssr::page::Page;

use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::response::Response;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::shared;
use crate::templates::AdminTemplateEngine;
use crate::types::{ENTITY_SKILL, MarketplaceContext, UserContext};

use super::entries::entry_columns;
use super::sorting::{apply_direction, direction, matches, preserved_search, sort_headers};
use super::view::assignment_counts_by_type;
use super::view_models::VisibilityInput;
use crate::handlers::ssr::ssr_helpers::render_typed_page;
use crate::handlers::ssr::types::BreadcrumbView;

use super::{CatalogListQuery, admin_only, visibility_inputs};

pub(crate) async fn skills_page(
    shell: Page,
    Extension(env): Extension<AdminEnv>,
    State(pool): State<Arc<PgPool>>,
    axum::extract::Query(query): axum::extract::Query<CatalogListQuery>,
) -> AdminHtmlResult<Response> {
    let Page {
        engine,
        user: user_ctx,
        marketplace: mkt_ctx,
    } = shell;
    admin_only(&user_ctx)?;
    let path = shared::get_services_path(&env);

    let catalog = super::data::load_catalog(&env, &path, &user_ctx.roles);
    let counts = assignment_counts_by_type(&pool, ENTITY_SKILL).await;
    let (manifests, rules) = visibility_inputs(&pool, &path).await;
    let mut skills = super::view_models::skill_rows(
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

    let page = super::view::SkillsPageData {
        page: "skills",
        title: "Skills",
        subtitle: "The instruction sets people invoke. A skill reaches someone through the plugins that include it.",
        breadcrumbs: vec![BreadcrumbView::current("Skills")],
        kpis: super::entries::skill_kpis(&skills, manifests.len()),
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

#[expect(
    clippy::too_many_arguments,
    reason = "axum extractor list; the router decides the arity, not this signature"
)]
pub(crate) async fn skill_detail_page(
    Extension(env): Extension<AdminEnv>,
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(skill_id): Path<String>,
    axum::extract::Query(query): axum::extract::Query<super::view::PanelQuery>,
) -> AdminHtmlResult<Response> {
    admin_only(&user_ctx)?;
    let path = shared::get_services_path(&env);

    let catalog = super::data::load_catalog(&env, &path, &user_ctx.roles);
    let counts = assignment_counts_by_type(&pool, ENTITY_SKILL).await;
    let assignment_count = counts.get(&skill_id).copied().unwrap_or(0);
    let skill = systemprompt::identifiers::SkillId::new(&skill_id);
    let mut page = super::view_models::skill_detail(&catalog, &skill, assignment_count)
        .ok_or_else(|| AdminError::NotFound("No such skill.".to_owned()))?;
    page.access = Some(
        super::access::panel(
            &env,
            &pool,
            &user_ctx,
            (ENTITY_SKILL, skill_id.as_str()),
            query.why.as_deref(),
        )
        .await,
    );
    Ok(render_typed_page(
        &engine,
        "catalog-skill-detail",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}
