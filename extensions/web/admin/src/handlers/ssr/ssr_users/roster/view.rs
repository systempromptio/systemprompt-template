//! Turning roster rows into the context the template renders.
//!
//! Every link on the page is built from the same preserved query string, so
//! sorting keeps the filter, filtering keeps the sort, and paging keeps both.

use std::collections::BTreeMap;

use crate::handlers::ssr::format::{format_cost, format_token_total, relative_time};
use crate::handlers::ssr::list_view::{PageWindow, Pagination};
use crate::repositories::users::roster::{RosterFilter, RosterRow, RosterStats};

use super::super::BASE_URL;
use super::RosterUrlState;
use crate::handlers::ssr::entity_urls::context_detail_url;
use crate::handlers::ssr::types::FilterChipView;

use super::context::{RosterChipView, RosterKpiView, RosterRowView};

pub(super) fn build_chips(url: &RosterUrlState, totals: &RosterStats) -> Vec<FilterChipView> {
    let prefix = url.link_prefix(&["filter", "page"]);
    let active = url.filter;
    vec![
        FilterChipView {
            label: "All",
            href: format!("{prefix}filter="),
            is_active: active == RosterFilter::None,
            count: totals.total,
        },
        FilterChipView {
            label: "Unassigned",
            href: format!("{prefix}filter=unassigned"),
            is_active: active == RosterFilter::Unassigned,
            count: totals.unassigned,
        },
        FilterChipView {
            label: "No role",
            href: format!("{prefix}filter=no-role"),
            is_active: active == RosterFilter::NoRole,
            count: totals.no_role,
        },
        FilterChipView {
            label: "Inactive 30d",
            href: format!("{prefix}filter=inactive-30d"),
            is_active: active == RosterFilter::Inactive30d,
            count: totals.inactive_30d,
        },
    ]
}

pub(super) fn build_kpis(url: &RosterUrlState, totals: &RosterStats) -> RosterKpiView {
    let prefix = url.link_prefix(&["filter", "page"]);
    let (cost_delta, cost_delta_dir) =
        delta(totals.cost_microdollars, totals.prior_cost_microdollars);
    let (requests_delta, requests_delta_dir) = delta(totals.requests, totals.prior_requests);
    RosterKpiView {
        total: totals.total,
        active: totals.active,
        unassigned: totals.unassigned,
        no_role: totals.no_role,
        inactive: totals.inactive_30d,
        cost_display: format_cost(totals.cost_microdollars),
        cost_delta,
        cost_delta_dir,
        requests_display: format_token_total(totals.requests),
        requests_delta,
        requests_delta_dir,
        unassigned_url: format!("{prefix}filter=unassigned"),
        no_role_url: format!("{prefix}filter=no-role"),
        inactive_url: format!("{prefix}filter=inactive-30d"),
        all_url: format!("{prefix}filter="),
        filter_active: url.filter != RosterFilter::None,
        unassigned_active: url.filter == RosterFilter::Unassigned,
        no_role_active: url.filter == RosterFilter::NoRole,
        inactive_active: url.filter == RosterFilter::Inactive30d,
    }
}

// Why: a percentage against the previous window of the same length. With no
// prior traffic there is nothing to compare, so the tile shows no delta rather
// than an infinite rise.
fn delta(current: i64, prior: i64) -> (String, &'static str) {
    if prior == 0 {
        return (String::new(), "flat");
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "a display percentage; the magnitudes here are far below f64's exact range"
    )]
    let pct = ((current - prior) as f64 / prior as f64) * 100.0;
    let dir = if pct > 1.0 {
        "up"
    } else if pct < -1.0 {
        "down"
    } else {
        "flat"
    };
    (format!("{pct:+.0}%"), dir)
}

pub(super) struct RowInput<'a> {
    pub rows: &'a [RosterRow],
    pub group_names: &'a BTreeMap<String, String>,
    pub project_names: &'a BTreeMap<String, String>,
}

pub(super) fn build_rows(input: &RowInput<'_>) -> Vec<RosterRowView> {
    input
        .rows
        .iter()
        .map(|row| {
            let name = row
                .display_name
                .clone()
                .unwrap_or_else(|| row.user_id.as_str().to_owned());
            RosterRowView {
                initials: initials(&name),
                detail_url: format!("/admin/users/{}", urlencoding::encode(row.user_id.as_str())),
                email: row
                    .email
                    .as_ref()
                    .map(|e| e.as_str().to_owned())
                    .unwrap_or_default(),
                has_roles: !row.roles.is_empty(),
                roles_title: row.roles.join(", "),
                roles: row.roles.clone(),
                has_scope: !row.group_ids.is_empty() || !row.project_ids.is_empty(),
                scope: chips(&row.group_ids, input.group_names, "group", "info")
                    .into_iter()
                    .chain(chips(
                        &row.project_ids,
                        input.project_names,
                        "project",
                        "accent",
                    ))
                    .collect(),
                last_active: row.last_active.map(relative_time),
                last_active_title: last_active_title(row),
                last_active_url: row.last_context_id.as_ref().map(context_detail_url),
                cost_display: format_cost(row.cost_microdollars),
                requests_display: format_token_total(row.requests),
                tokens_display: format_token_total(row.tokens),
                status_label: if row.is_active { "Active" } else { "Suspended" },
                status_tone: if row.is_active { "ok" } else { "muted" },
                is_active: row.is_active,
                name,
                user_id: row.user_id.clone(),
            }
        })
        .collect()
}

fn chips(
    ids: &[String],
    names: &BTreeMap<String, String>,
    facet: &str,
    tone: &'static str,
) -> Vec<RosterChipView> {
    ids.iter()
        .map(|id| RosterChipView {
            label: names.get(id).cloned().unwrap_or_else(|| id.clone()),
            href: format!("{BASE_URL}?{facet}={}", urlencoding::encode(id)),
            tone,
            id: id.clone(),
        })
        .collect()
}

// Why: the stamp alone cannot say which of the three activity clocks it came
// from, and an admin reading "2h ago" on an account with no gateway traffic
// needs to know it was a sign-in, not a prompt.
fn last_active_title(row: &RosterRow) -> String {
    let Some(t) = row.last_active else {
        return "No recorded activity".to_owned();
    };
    let source = match row.last_active_source.as_deref() {
        Some("gateway") => "last gateway request",
        Some("session") => "last sign-in activity",
        Some("console") => "last console action",
        _ => "last activity",
    };
    format!("{} · {source}", t.to_rfc3339())
}

fn initials(name: &str) -> String {
    name.split(|c: char| c.is_whitespace() || c == '-' || c == '.' || c == '@')
        .filter(|part| !part.is_empty())
        .take(2)
        .filter_map(|part| part.chars().next())
        .flat_map(char::to_uppercase)
        .collect()
}

pub(super) fn build_pagination(url: &RosterUrlState, window: PageWindow) -> Pagination {
    let page = window.index;
    let prefix = url.link_prefix(&["page"]);
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
