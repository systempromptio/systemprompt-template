//! The `/admin/projects` listing: read the rollups, then order and page them.
//!
//! Ordering happens in Rust rather than in SQL because the listing reads every
//! project in one pass — an instance has tens of projects, not thousands, and
//! one static statement that cannot be reordered by a query parameter is worth
//! more than saving a sort over a few hundred rows.

use sqlx::PgPool;

use crate::repositories::people_usage::{DEFAULT_WINDOW_DAYS, totals};
use crate::repositories::projects::usage::{LISTING_CAP, ProjectRollup, list_project_rollups};
use crate::repositories::scope::membership::UNATTRIBUTED;
use crate::repositories::scope::{Attribution, ScopeKind};

use super::super::people_view::{format_usd, or_default};
use super::super::types::{BreadcrumbView, ProjectKpiView, ProjectListRowView, ProjectsPageData};
use super::sort::{page_url, sort_headers, sort_key, sort_rows};
use super::{PAGE_SIZE, WINDOW_LABEL, pagination, pct};

// Why: what the query string may ask of the listing.
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct ListQuery {
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub page: Option<i64>,
    pub q: Option<String>,
}

pub(super) async fn page_data(
    pool: &PgPool,
    query: &ListQuery,
    can_manage: bool,
) -> ProjectsPageData {
    let all = or_default(
        "project rollups",
        list_project_rollups(pool, DEFAULT_WINDOW_DAYS, LISTING_CAP).await,
    );
    let unattributed = unattributed_requests(pool).await;
    let kpis = kpis(&all, unattributed);
    let truncated = all.len() as i64 >= LISTING_CAP;

    let needle = query.q.as_deref().unwrap_or_default().trim().to_lowercase();
    let mut matched: Vec<ProjectRollup> = all
        .into_iter()
        .filter(|p| {
            needle.is_empty()
                || p.name.to_lowercase().contains(&needle)
                || p.id.to_lowercase().contains(&needle)
        })
        .collect();

    let sort = sort_key(query.sort.as_deref());
    let descending = query.dir.as_deref() != Some("asc");
    sort_rows(&mut matched, sort, descending);

    let total = matched.len() as i64;
    let page = query.page.unwrap_or(1).max(1);
    let offset = ((page - 1) * PAGE_SIZE).min(total.max(0));
    let rows: Vec<ProjectListRowView> = matched
        .iter()
        .skip(usize::try_from(offset).unwrap_or(0))
        .take(usize::try_from(PAGE_SIZE).unwrap_or(50))
        .map(row_view)
        .collect();

    ProjectsPageData {
        page: "projects",
        title: "Projects",
        breadcrumbs: vec![BreadcrumbView::current("Projects")],
        subtitle: "Work attribution. A project says what people are building; a group says what they may reach.",
        window_label: WINDOW_LABEL.to_owned(),
        kpis,
        sort_headers: sort_headers(query, sort, descending),
        count_label: count_label(total, &needle),
        pagination: pagination(
            page,
            total,
            offset,
            rows.len() as i64,
            "projects",
            &page_url(query),
        ),
        rows,
        query: query.q.clone().unwrap_or_default(),
        truncated,
        can_manage,
    }
}

async fn unattributed_requests(pool: &PgPool) -> i64 {
    or_default(
        "project scope totals",
        totals::list_scope_totals(
            pool,
            ScopeKind::Project,
            Attribution::Exclusive,
            DEFAULT_WINDOW_DAYS,
        )
        .await,
    )
    .into_iter()
    .find(|row| row.scope_id == UNATTRIBUTED)
    .map_or(0, |row| row.requests)
}

fn count_label(total: i64, needle: &str) -> String {
    if needle.is_empty() {
        format!("{total} projects")
    } else {
        format!("{total} projects matching “{needle}”")
    }
}

fn kpis(rows: &[ProjectRollup], unattributed: i64) -> Vec<ProjectKpiView> {
    let members: i64 = rows.iter().map(|r| r.member_count).sum();
    let active: i64 = rows.iter().map(|r| r.active_members).sum();
    let requests: i64 = rows.iter().map(|r| r.requests).sum();
    let cost: i64 = rows.iter().map(|r| r.cost_microdollars).sum();
    let calls: i64 = rows.iter().map(|r| r.tool_calls).sum();
    let ok: i64 = rows.iter().map(|r| r.tool_success).sum();
    let success = pct(ok, calls);
    vec![
        tile(
            "Projects",
            rows.len().to_string(),
            "work buckets on this instance",
            "accent",
        ),
        tile(
            "Members",
            members.to_string(),
            format!("{active} active in the window"),
            "accent",
        ),
        tile(
            "Requests",
            requests.to_string(),
            "attributed to a project",
            "accent",
        ),
        tile(
            "Cost",
            format_usd(cost),
            "billed across every project",
            "accent",
        ),
        tile(
            "Tool success",
            format!("{success}%"),
            format!("{ok} of {calls} MCP calls"),
            tool_tone(success, calls),
        ),
        tile(
            "Unattributed",
            unattributed.to_string(),
            "requests no project claims",
            if unattributed > 0 { "warn" } else { "ok" },
        ),
    ]
}

fn tile(label: &str, value: String, note: impl Into<String>, tone: &'static str) -> ProjectKpiView {
    ProjectKpiView {
        label: label.to_owned(),
        value,
        note: note.into(),
        tone,
        href: None,
    }
}

// Why: a success rate is only a signal once there are calls behind it; a
// project with none reads as neutral rather than as a perfect score.
const fn tool_tone(success: i64, calls: i64) -> &'static str {
    if calls == 0 {
        "accent"
    } else if success >= 95 {
        "ok"
    } else if success >= 80 {
        "warn"
    } else {
        "err"
    }
}

fn row_view(p: &ProjectRollup) -> ProjectListRowView {
    let success = pct(p.tool_success, p.tool_calls);
    ProjectListRowView {
        href: format!("/admin/projects/{}", p.id),
        id: p.id.clone(),
        name: p.name.clone(),
        description: p.description.clone(),
        member_count: p.member_count,
        active_members: p.active_members,
        group_count: p.group_count,
        requests: p.requests,
        cost_display: format_usd(p.cost_microdollars),
        tool_calls: p.tool_calls,
        tool_success_pct: success,
        tool_tone: tool_tone(success, p.tool_calls),
        skills_used: p.skills_used,
    }
}
