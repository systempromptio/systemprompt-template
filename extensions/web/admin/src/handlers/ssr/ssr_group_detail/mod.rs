//! `/admin/groups/{group_id}` — one group across five tabs, and the
//! Unassigned bucket as a variant of the same page.
//!
//! Unassigned is not a group anyone created: it is everyone with no group row
//! at all, so it has no rules and no directory mapping of its own. The page
//! renders it with a callout and an assign control instead of the stat row and
//! the Access and Mappings tabs, because those would edit a subject that can
//! never hold a rule.

use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::people_chart::daily_requests_chart;
use super::people_view;
use super::ssr_groups::UNASSIGNED_GROUP;
use super::types::{
    BreadcrumbView, GroupDetailPageData, GroupOverviewView, MarketplaceAssignmentView,
    MemberSetChipView, MembersTabView, UsageLeaderRowView,
};

mod data;
mod tabs;
mod view;

#[derive(Debug, Deserialize)]
pub(crate) struct TabQuery {
    tab: Option<String>,
}

#[expect(
    clippy::too_many_arguments,
    reason = "axum extractor list; the router decides the arity, not this signature"
)]
pub(crate) async fn group_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(group_id): Path<String>,
    Query(query): Query<TabQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let group = repositories::groups::crud::find_group(&pool, &group_id).await?;
    let is_unassigned = group_id == UNASSIGNED_GROUP;
    let page = if is_unassigned {
        "unassigned"
    } else {
        "group-detail"
    };

    let Some(group) = group else {
        return Err(AdminError::NotFound("No such group.".to_owned()).into());
    };

    let active = tabs::resolve_tab(query.tab.as_deref(), is_unassigned);
    let counts = data::load_counts(&pool, &group_id).await;
    let usage = data::load_usage(&pool, &group_id).await;

    let data = GroupDetailPageData {
        page,
        title: group.name.clone(),
        breadcrumbs: breadcrumbs(&group.name),
        tabs: tabs::tab_links(&group_id, active, is_unassigned, &counts),
        stats: if is_unassigned {
            Vec::new()
        } else {
            people_view::stat_tiles(&usage, counts.members)
        },
        overview: load_overview(&pool, &group_id, active).await,
        members: load_members(&pool, &group_id, active, &user_ctx).await,
        marketplaces: load_marketplaces(&pool, &group_id, active).await,
        access: load_access(&pool, &group_id, active).await,
        projects: load_projects(&pool, &group_id, active).await,
        mappings: load_mappings(&pool, &group_id, active, &user_ctx).await,
        active_tab: active.to_owned(),
        group_name: group.name,
        description: group.description,
        is_unassigned,
        not_found: false,
        can_manage: user_ctx.is_admin,
        can_map: crate::types::roles_grant_platform(&user_ctx.roles),
        group_id,
    };

    Ok(super::render_typed_page(
        &engine,
        "group-detail",
        &data,
        &user_ctx,
        &mkt_ctx,
    ))
}

async fn load_overview(pool: &PgPool, group_id: &str, active: &str) -> Option<GroupOverviewView> {
    if active != tabs::USAGE {
        return None;
    }
    let loaded = data::load_usage_tab(pool, group_id).await;
    let top = loaded
        .leaderboard
        .first()
        .map_or(0, |r| r.cost_microdollars);
    Some(GroupOverviewView {
        models: people_view::model_rows(&loaded.models),
        daily_requests: daily_requests_chart(&loaded.daily),
        skills: people_view::name_count_rows(&loaded.skills),
        tools: people_view::name_count_rows(&loaded.tools),
        leaderboard: loaded
            .leaderboard
            .iter()
            .map(|r| UsageLeaderRowView {
                href: format!("/admin/users/{}", urlencoding::encode(r.user_id.as_str())),
                user_id: r.user_id.clone(),
                requests: r.requests,
                tokens: r.tokens,
                cost_microdollars: r.cost_microdollars,
                share_pct: people_view::share(r.cost_microdollars, top),
            })
            .collect(),
    })
}

// Why: the whole catalog, flagged with what this group already reaches. The
// rows a group is not entitled to are what makes the tab an editor rather
// than a receipt.
async fn load_marketplaces(
    pool: &PgPool,
    group_id: &str,
    active: &str,
) -> Option<Vec<MarketplaceAssignmentView>> {
    if active != tabs::MARKETPLACES {
        return None;
    }
    Some(
        data::load_marketplace_options(pool, group_id)
            .await
            .into_iter()
            .map(|m| MarketplaceAssignmentView {
                id: m.id,
                name: m.name,
                description: m.description,
                assigned: m.assigned,
            })
            .collect(),
    )
}

async fn load_members(
    pool: &PgPool,
    group_id: &str,
    active: &str,
    user_ctx: &UserContext,
) -> Option<MembersTabView> {
    if active != tabs::MEMBERS {
        return None;
    }
    let loaded = data::load_members(pool, group_id, user_ctx.is_admin).await;
    let assign_targets = if group_id == UNASSIGNED_GROUP && user_ctx.is_admin {
        assign_targets(pool).await
    } else {
        Vec::new()
    };
    Some(MembersTabView {
        rows: view::group_member_rows(&loaded, user_ctx.is_admin),
        addable_users: loaded.addable,
        assign_targets,
    })
}

// Why: Destinations for the Unassigned page's assign control: every real group.
async fn assign_targets(pool: &PgPool) -> Vec<MemberSetChipView> {
    repositories::groups::crud::list_groups(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "assign target list failed"))
        .unwrap_or_default()
        .into_iter()
        .filter(|g| g.id != UNASSIGNED_GROUP)
        .map(|g| MemberSetChipView {
            id: g.id,
            label: g.name,
        })
        .collect()
}

async fn load_access(
    pool: &PgPool,
    group_id: &str,
    active: &str,
) -> Option<Vec<super::types::AccessSectionView>> {
    if active != tabs::ACCESS {
        return None;
    }
    Some(data::load_access(pool, group_id).await)
}

async fn load_projects(
    pool: &PgPool,
    group_id: &str,
    active: &str,
) -> Option<Vec<super::types::ProjectRowView>> {
    if active != tabs::PROJECTS {
        return None;
    }
    let rows = data::load_projects(pool, group_id).await;
    Some(people_view::linked_rows(&rows, "projects"))
}

async fn load_mappings(
    pool: &PgPool,
    group_id: &str,
    active: &str,
    user_ctx: &UserContext,
) -> Option<Vec<super::types::MappingRowView>> {
    if active != tabs::MAPPINGS {
        return None;
    }
    let rows = data::load_mappings(pool, group_id).await;
    Some(people_view::mapping_rows(
        rows.into_iter().map(|r| (r.ad_group, r.source)),
        user_ctx.is_platform_admin,
    ))
}

fn breadcrumbs(name: &str) -> Vec<BreadcrumbView> {
    vec![
        BreadcrumbView::link("Groups", "/admin/groups"),
        BreadcrumbView::current(name),
    ]
}
