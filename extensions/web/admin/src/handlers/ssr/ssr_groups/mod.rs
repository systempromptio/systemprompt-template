//! `/admin/groups` — every group, with its membership, entitlement and spend.
//!
//! `unassigned` is a row here like any other. It is derived rather than
//! stored — anyone with no `group_members` row falls into it — and the whole
//! point of the page is that the number next to it is visible rather than
//! implicit, so it is listed and badged, never filtered out.
//!
//! The spend columns are exclusively attributed, so the rows plus the
//! unattributed row at the foot of the table reproduce the instance total.
//! That is the property the page is built around: a group listing whose
//! numbers do not add up to anything cannot answer "where did the money go".

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::types::{BreadcrumbView, GroupsPageData, MemberSetChipView};

mod data;
mod sorting;
mod view;

pub(crate) const UNASSIGNED_GROUP: &str = "unassigned";

#[derive(Debug, Deserialize)]
pub(crate) struct GroupsQuery {
    range: Option<String>,
    sort: Option<String>,
    dir: Option<String>,
    page: Option<i64>,
    source: Option<String>,
}

pub(crate) async fn groups_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<GroupsQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let window_days = sorting::window_days(query.range.as_deref());
    let range = query.range.as_deref().unwrap_or("30d").to_owned();
    let sort = sorting::sort_key(query.sort.as_deref());
    let dir = sorting::direction(query.dir.as_deref());

    let source = sorting::source_filter(query.source.as_deref());
    let mut listing = data::load_listing(&pool, window_days).await;
    sorting::apply(&mut listing.rows, sort, dir);

    // Why: the tiles are computed before the source filter is applied. They
    // report the estate, and the whole point of the exclusive split is that
    // they reconcile with it — narrowing them to the rows on screen would give
    // a "Cost" that means something different on every filtered view.
    let range_label = sorting::range_label(window_days);
    let kpis = view::kpis(
        &listing.rows,
        &listing.unattributed,
        listing.unkeyed_people,
        range_label,
    );

    if !source.is_empty() {
        listing.rows.retain(|r| r.source_label == source);
    }
    let total_groups = listing.rows.len() as i64;
    let (rows, pagination) = view::paginate(
        listing.rows,
        query.page.unwrap_or(1),
        &range,
        sort,
        dir,
        &source,
    );

    let data = GroupsPageData {
        page: "groups",
        title: "Groups",
        breadcrumbs: vec![BreadcrumbView::current("Groups")],
        kpis,
        sort_headers: sorting::sort_headers(sort, dir, &range, &source),
        ranges: sorting::range_links(window_days, &source),
        sources: sorting::source_links(&source, &range),
        source_filter: source.clone(),
        range_label,
        toolbar_count: format!("{total_groups} groups · last {range_label}"),
        window_days,
        groups: rows,
        unattributed: listing.unattributed,
        pagination,
        total_groups,
        can_manage: user_ctx.is_admin,
        can_map: crate::types::roles_grant_platform(&user_ctx.roles),
        group_options: group_options(&pool).await,
        unkeyed_people: listing.unkeyed_people,
    };

    Ok(super::render_typed_page(
        &engine, "groups", &data, &user_ctx, &mkt_ctx,
    ))
}

// Why: the destination select on the header's mapping dialog. Unassigned is
// excluded: it is where people land when no mapping matched, so a directory
// group mapped into it would be a contradiction.
async fn group_options(pool: &PgPool) -> Vec<MemberSetChipView> {
    repositories::groups::crud::list_groups(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "group option list failed"))
        .unwrap_or_default()
        .into_iter()
        .filter(|g| g.id != UNASSIGNED_GROUP)
        .map(|g| MemberSetChipView {
            id: g.id,
            label: g.name,
        })
        .collect()
}
