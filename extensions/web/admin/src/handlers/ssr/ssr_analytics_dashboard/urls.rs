//! URL builders for the site analytics dashboard.
//!
//! Every link the page emits — tab bar, filter chips, bucket toggle, sort
//! headers, pagination, and the leaderboard's self-drill scope links —
//! rebuilds the current query string minus the parameter it is changing, so
//! the reader never loses the window or the filters they picked.

use urlencoding::encode as urlencode;

use crate::handlers::ssr::list_view::{PageWindow, Pagination};

use super::context::{BucketLinkView, DashboardTab, DashboardTabLink, ScopeChipView, SloOption};

pub(super) use super::urls_controls::{
    attribution_links, audience_links, axis_links, cost_csv_url, drill_url,
};
use super::{AnalyticsDashboardQuery, BASE_URL};

pub(super) fn preserved_query_string(query: &AnalyticsDashboardQuery, drop: &[&str]) -> String {
    let mut parts: Vec<String> = Vec::new();
    let pairs: [(&str, Option<&str>); 12] = [
        ("tab", query.tab.as_deref()),
        ("preset", query.preset.as_deref()),
        ("from", query.from.as_deref()),
        ("to", query.to.as_deref()),
        ("bucket", query.bucket.as_deref()),
        ("group", query.group.as_deref()),
        ("project", query.project.as_deref()),
        (
            "user_id",
            query
                .user_id
                .as_ref()
                .map(systemprompt::identifiers::UserId::as_str),
        ),
        ("sort", query.sort.as_deref()),
        ("attr", query.attr.as_deref()),
        ("audience", query.audience.as_deref()),
        ("axis", query.axis.as_deref()),
    ];
    for (name, value) in pairs {
        if drop.contains(&name) {
            continue;
        }
        let Some(v) = value.filter(|s| !s.is_empty()) else {
            continue;
        };
        parts.push(format!("{}={}", name, urlencode(v)));
    }
    if !drop.contains(&"page")
        && let Some(p) = query.page.filter(|p| *p > 0)
    {
        parts.push(format!("page={p}"));
    }
    if !drop.contains(&"slo_ms")
        && let Some(ms) = query.slo_ms
    {
        parts.push(format!("slo_ms={ms}"));
    }
    parts.join("&")
}

pub(super) fn with_qs(base: String, qs: &str) -> String {
    if qs.is_empty() {
        base
    } else if base.contains('?') {
        format!("{base}&{qs}")
    } else {
        format!("{base}?{qs}")
    }
}

pub(super) fn tab_links(
    active: DashboardTab,
    query: &AnalyticsDashboardQuery,
) -> Vec<DashboardTabLink> {
    const TABS: [(DashboardTab, &str); 6] = [
        (DashboardTab::Overview, "Overview"),
        (DashboardTab::Models, "Models"),
        (DashboardTab::Skills, "Skills"),
        (DashboardTab::Tools, "Tools"),
        (DashboardTab::Sessions, "Sessions"),
        (DashboardTab::Cost, "Cost"),
    ];
    let qs = preserved_query_string(query, &["tab", "page", "sort"]);
    TABS.iter()
        .map(|&(tab, label)| DashboardTabLink {
            slug: tab.as_str(),
            label,
            href: with_qs(format!("{BASE_URL}?tab={}", tab.as_str()), &qs),
            is_active: tab == active,
        })
        .collect()
}

pub(super) fn bucket_links(
    query: &AnalyticsDashboardQuery,
    active_week: bool,
) -> Vec<BucketLinkView> {
    let qs = preserved_query_string(query, &["bucket", "page"]);
    vec![
        BucketLinkView {
            label: "Daily",
            href: with_qs(format!("{BASE_URL}?bucket=day"), &qs),
            is_active: !active_week,
        },
        BucketLinkView {
            label: "Weekly",
            href: with_qs(format!("{BASE_URL}?bucket=week"), &qs),
            is_active: active_week,
        },
    ]
}

pub(super) fn active_chips(query: &AnalyticsDashboardQuery) -> Vec<ScopeChipView> {
    let mut chips = Vec::new();
    let candidates: [(&str, &'static str, Option<&str>); 3] = [
        ("group", "Group", query.group.as_deref()),
        ("project", "Project", query.project.as_deref()),
        (
            "user_id",
            "User",
            query
                .user_id
                .as_ref()
                .map(systemprompt::identifiers::UserId::as_str),
        ),
    ];
    for (param, group_label, value) in candidates {
        let Some(value) = value.filter(|s| !s.is_empty()) else {
            continue;
        };
        let qs = preserved_query_string(query, &[param, "page"]);
        chips.push(ScopeChipView {
            group_label,
            label: value.to_owned(),
            remove_url: with_qs(BASE_URL.to_owned(), &qs),
        });
    }
    chips
}

pub(super) fn clear_url(query: &AnalyticsDashboardQuery) -> String {
    let qs = preserved_query_string(query, &["project", "user_id", "page", "sort"]);
    with_qs(BASE_URL.to_owned(), &qs)
}

// Why: the dashboard re-rendered scoped to one user — the "Focus" drill.
pub(super) fn scope_to_user_url(
    query: &AnalyticsDashboardQuery,
    user_id: &systemprompt::identifiers::UserId,
) -> String {
    let qs = preserved_query_string(query, &["user_id", "page"]);
    with_qs(
        format!("{BASE_URL}?user_id={}", urlencode(user_id.as_str())),
        &qs,
    )
}

pub(super) fn sort_url(query: &AnalyticsDashboardQuery, sort: &str) -> String {
    let qs = preserved_query_string(query, &["sort", "page"]);
    with_qs(format!("{BASE_URL}?sort={sort}"), &qs)
}

pub(super) fn build_pagination(query: &AnalyticsDashboardQuery, window: PageWindow) -> Pagination {
    let page = window.index;
    let qs = preserved_query_string(query, &["page"]);
    let prefix = if qs.is_empty() {
        format!("{BASE_URL}?")
    } else {
        format!("{BASE_URL}?{qs}&")
    };
    let prev_url = (page > 0).then(|| format!("{prefix}page={}", page - 1));
    let next_url = (page + 1 < window.total_pages).then(|| format!("{prefix}page={}", page + 1));
    let (first_row, last_row) = window.bounds();
    Pagination {
        current_page: page + 1,
        total_pages: window.total_pages,
        first_row,
        last_row,
        total_rows: window.total_rows,
        noun: window.noun,
        has_prev: prev_url.is_some(),
        has_next: next_url.is_some(),
        prev_url,
        next_url,
    }
}

// Why: same link-not-select shape as the inactivity window below, for the
// same reason: the SLO threshold only means anything on the Spend tab.
pub(super) fn slo_links(query: &AnalyticsDashboardQuery, active_ms: i32) -> Vec<SloOption> {
    const CHOICES: [(i32, &str); 4] =
        [(1_000, "1s"), (2_000, "2s"), (5_000, "5s"), (10_000, "10s")];
    let qs = preserved_query_string(query, &["slo_ms", "page"]);
    CHOICES
        .iter()
        .map(|&(ms, label)| SloOption {
            label: label.to_owned(),
            href: with_qs(format!("{BASE_URL}?slo_ms={ms}"), &qs),
            selected: ms == active_ms,
        })
        .collect()
}
