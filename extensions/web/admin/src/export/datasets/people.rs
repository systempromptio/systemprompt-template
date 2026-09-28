//! The users roster and the groups listing as files.
//!
//! Both read through the same repository calls the pages make, under the
//! same filters the page URL carries, so the file answers the question the
//! screen was asked. The roster is the one export here that pages, so it
//! walks the repository page by page up to the cap.

use async_trait::async_trait;
use serde::Deserialize;

use crate::error::AdminResult;
use crate::export::model::{Cell, CellKind, Column, DataSet, ExportContext, Table, Window};
use crate::repositories::groups::usage::{GroupUsageRow, list_groups_with_usage};
use crate::repositories::scope::ScopeRequest;
use crate::repositories::scope::membership::get_subject_scope;
use crate::repositories::users::roster::{
    RosterFilter, RosterQuery, RosterRow, RosterSort, list_users_paged,
};

pub(crate) struct Users;
pub(crate) struct Groups;

#[derive(Debug, Default, Deserialize)]
struct UsersQuery {
    filter: Option<String>,
    role: Option<String>,
    q: Option<String>,
    group: Option<String>,
    project: Option<String>,
    sort: Option<String>,
    dir: Option<String>,
}

const PAGE: i64 = 1_000;

const USER_COLUMNS: &[Column] = &[
    Column::new("user_id", "User id", CellKind::Text).group("Identity"),
    Column::new("name", "Name", CellKind::Text).group("Identity"),
    Column::new("email", "Email", CellKind::Text).group("Identity"),
    Column::new("roles", "Roles", CellKind::Text).group("Access"),
    Column::new("groups", "Groups", CellKind::Text).group("Access"),
    Column::new("projects", "Projects", CellKind::Text).group("Access"),
    Column::new("status", "Status", CellKind::Text).group("Access"),
    Column::new("created_at", "Created", CellKind::Timestamp).group("Activity"),
    Column::new("last_active", "Last active", CellKind::Timestamp).group("Activity"),
    Column::new("last_active_source", "Seen via", CellKind::Text)
        .group("Activity")
        .optional(),
    Column::new("requests", "Requests (30d)", CellKind::Integer).group("Usage"),
    Column::new("tokens", "Tokens (30d)", CellKind::Integer).group("Usage"),
    Column::new("cost", "Cost (30d)", CellKind::Money).group("Usage"),
];

fn user_row(r: &RosterRow) -> Vec<Cell> {
    vec![
        r.user_id.as_str().into(),
        Cell::opt_text(r.display_name.as_deref()),
        Cell::opt_text(r.email.as_ref().map(ToString::to_string)),
        Cell::list(&r.roles),
        Cell::list(&r.group_ids),
        Cell::list(&r.project_ids),
        if r.is_active { "active" } else { "disabled" }.into(),
        r.created_at.into(),
        Cell::opt_time(r.last_active),
        Cell::opt_text(r.last_active_source.as_deref()),
        r.requests.into(),
        r.tokens.into(),
        Cell::Money(r.cost_microdollars),
    ]
}

#[async_trait]
impl DataSet for Users {
    fn id(&self) -> &'static str {
        "users"
    }
    fn title(&self) -> &'static str {
        "Users"
    }
    fn description(&self) -> &'static str {
        "One row per account: identity, roles, groups, projects, when they were last seen and their thirty-day gateway spend."
    }
    fn columns(&self) -> &'static [Column] {
        USER_COLUMNS
    }
    fn window(&self) -> Window {
        Window::None
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let q: UsersQuery = ctx.query()?;
        let request = ScopeRequest::from_query(ctx.user, q.group.as_deref(), q.project.as_deref());
        let scope = get_subject_scope(ctx.pool, &request).await?;
        let mut query = RosterQuery {
            filter: RosterFilter::parse_filter(q.filter.as_deref()),
            role: q.role.filter(|r| !r.is_empty()),
            search: q.q.filter(|s| !s.is_empty()),
            sort: RosterSort::parse_sort(q.sort.as_deref(), q.dir.as_deref()),
            limit: PAGE,
            offset: 0,
        };
        let mut rows = Vec::new();
        let total = loop {
            let (page, total) = list_users_paged(ctx.pool, &scope, &query).await?;
            let last = (page.len() as i64) < PAGE;
            rows.extend(page.iter().map(user_row));
            query.offset += PAGE;
            if last || query.offset >= ctx.limit {
                break total;
            }
        };
        Ok(Table { rows, total })
    }
}

const GROUP_COLUMNS: &[Column] = &[
    Column::new("id", "Group id", CellKind::Text).group("Identity"),
    Column::new("name", "Name", CellKind::Text).group("Identity"),
    Column::new("description", "Description", CellKind::Text)
        .group("Identity")
        .optional(),
    Column::new("source", "Source", CellKind::Text).group("Identity"),
    Column::new("is_system", "Built-in", CellKind::Bool).group("Identity"),
    Column::new("members", "Members", CellKind::Integer).group("People"),
    Column::new("active_members", "Active members", CellKind::Integer).group("People"),
    Column::new("projects", "Projects", CellKind::Integer).group("People"),
    Column::new("requests", "Requests", CellKind::Integer).group("Usage"),
    Column::new("tokens", "Tokens", CellKind::Integer).group("Usage"),
    Column::new("cost", "Cost", CellKind::Money).group("Usage"),
    Column::new("top_model", "Top model", CellKind::Text).group("Usage"),
    Column::new(
        "top_model_requests",
        "Top model requests",
        CellKind::Integer,
    )
    .group("Usage"),
];

fn group_row(g: &GroupUsageRow) -> Vec<Cell> {
    vec![
        g.id.as_str().into(),
        g.name.as_str().into(),
        Cell::opt_text(g.description.as_deref()),
        g.source.as_str().into(),
        g.is_system.into(),
        g.member_count.into(),
        g.active_members.into(),
        g.project_count.into(),
        g.requests.into(),
        g.tokens.into(),
        Cell::Money(g.cost_microdollars),
        Cell::opt_text(g.top_model.as_deref()),
        g.top_model_requests.into(),
    ]
}

#[async_trait]
impl DataSet for Groups {
    fn id(&self) -> &'static str {
        "groups"
    }
    fn title(&self) -> &'static str {
        "Groups"
    }
    fn description(&self) -> &'static str {
        "One row per group with the Unattributed remainder: membership, projects, and exclusively attributed requests, tokens and cost."
    }
    fn columns(&self) -> &'static [Column] {
        GROUP_COLUMNS
    }
    fn window(&self) -> Window {
        Window::Days
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let window = ctx.window()?;
        // Why: the rollup counts back from now in whole days, so the dialog's
        // day preset is the number of days it spans.
        let days = i32::try_from((window.to - window.from).num_days().max(1)).unwrap_or(30);
        let rows = list_groups_with_usage(ctx.pool, days).await?;
        Ok(Table::complete(rows.iter().map(group_row).collect()))
    }
}
