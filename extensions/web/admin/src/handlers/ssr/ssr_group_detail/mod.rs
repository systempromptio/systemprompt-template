//! `/admin/groups/{group_id}` — one group across five tabs (its marketplaces
//! are the first section of Access), and the
//! Unassigned bucket as a variant of the same page.
//!
//! Unassigned is not a group anyone created: it is everyone with no group row
//! at all, so it has no rules and no directory mapping of its own. The page
//! renders it with a callout and an assign control instead of the stat row and
//! the Access and Mappings tabs, because those would edit a subject that can
//! never hold a rule.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt_web_shared::GroupId;

use crate::error::{AdminError, AdminHtmlResult};
use crate::repositories;
use crate::types::UserContext;

use super::people_chart::daily_requests_chart;
use super::people_view;
use super::ssr_groups::UNASSIGNED_GROUP;
use super::types::{
    BreadcrumbView, GroupDetailPageData, GroupOverviewView, MemberSetChipView, MembersTabView,
    UsageLeaderRowView,
};
use crate::handlers::ssr::page::Page;

mod data;
mod tabs;
mod view;

#[derive(Debug, Deserialize)]
pub(crate) struct TabQuery {
    tab: Option<String>,
}

pub(crate) async fn group_detail_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(group_id): Path<GroupId>,
    Query(query): Query<TabQuery>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    // Why: a group's marketplaces are rows of its Access tab now; the old
    // tab's links land there.
    if query.tab.as_deref() == Some("marketplaces") {
        return Ok(Redirect::to(&format!("/admin/groups/{group_id}?tab=access")).into_response());
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
        members: load_members(&pool, &group_id, active, &shell.user).await,
        access: load_access(&pool, &group_id, active).await,
        projects: load_projects(&pool, &group_id, active).await,
        mappings: load_mappings(&pool, &group_id, active, &shell.user).await,
        active_tab: active.to_owned(),
        group_name: group.name,
        description: group.description,
        is_unassigned,
        not_found: false,
        can_manage: shell.user.is_admin,
        can_map: shell.user.is_platform_admin,
        export: (!is_unassigned).then(|| export_view(&group_id)),
        group_id,
    };

    Ok(super::render_typed_page(
        &shell.engine,
        "group-detail",
        &data,
        &shell.user,
        &shell.marketplace,
    ))
}

async fn load_overview(
    pool: &PgPool,
    group_id: &GroupId,
    active: &str,
) -> Option<GroupOverviewView> {
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

async fn load_members(
    pool: &PgPool,
    group_id: &GroupId,
    active: &str,
    user_ctx: &UserContext,
) -> Option<MembersTabView> {
    if active != tabs::MEMBERS {
        return None;
    }
    let loaded = data::load_members(pool, group_id, user_ctx.is_admin).await;
    let assign_targets = if group_id.as_str() == UNASSIGNED_GROUP && user_ctx.is_admin {
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
            id: g.id.as_str().to_owned(),
            label: g.name,
        })
        .collect()
}

async fn load_access(
    pool: &PgPool,
    group_id: &GroupId,
    active: &str,
) -> Option<Vec<super::types::AccessSectionView>> {
    if active != tabs::ACCESS {
        return None;
    }
    Some(data::load_access(pool, group_id).await)
}

async fn load_projects(
    pool: &PgPool,
    group_id: &GroupId,
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
    group_id: &GroupId,
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

// Why: the group's own traffic, keyed by `group` the way every scoped
// dataset reads it. Unassigned is membership by absence, which no `group`
// filter can name, so it offers no export.
fn export_view(group_id: &GroupId) -> crate::export::ExportView {
    let query = crate::export::view::query_string(&[("group", Some(group_id.as_str()))]);
    // Why: `analysis-conversations` and the transcript bundle join this list
    // with the analysis suite (Stage 3 phase 7); an id the registry does not
    // hold is dropped by `ExportView::new`.
    crate::export::ExportView::new(&["requests", "sessions", "analysis-conversations"], &query)
}

fn breadcrumbs(name: &str) -> Vec<BreadcrumbView> {
    vec![
        BreadcrumbView::link("Groups", "/admin/groups"),
        BreadcrumbView::current(name),
    ]
}
