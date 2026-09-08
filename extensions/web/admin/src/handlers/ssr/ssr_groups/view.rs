//! The group listing's header tiles and its page window.

use super::super::list_view::Pagination;
use super::super::people_view::format_usd;
use super::super::types::{GroupKpiView, GroupRowView, UnattributedRowView};
use super::UNASSIGNED_GROUP;
use super::sorting::BASE_URL;

// Why: what a page of groups holds. Fifty is the design system's default and
// no estate in this product has come near it, so paging is a guard rather
// than a routine step.
pub(super) const PAGE_SIZE: i64 = 50;

// Why: the KPI row answers the five questions this page exists for before any
// column is read: how many groups, how many people, what the estate spent,
// how much of that no group accounts for, and whether attribution is intact.
pub(super) fn kpis(
    rows: &[GroupRowView],
    unattributed: &UnattributedRowView,
    unkeyed_people: i64,
    range_label: &str,
) -> Vec<GroupKpiView> {
    let real: Vec<&GroupRowView> = rows.iter().filter(|r| !r.is_unassigned).collect();
    let members: i64 = real.iter().map(|r| r.member_count).sum();
    let unassigned = rows
        .iter()
        .find(|r| r.id == UNASSIGNED_GROUP)
        .map_or(0, |r| r.member_count);
    let requests: i64 = rows.iter().map(|r| r.requests).sum::<i64>() + unattributed.requests;
    let cost: i64 =
        rows.iter().map(|r| r.cost_microdollars).sum::<i64>() + unattributed.cost_microdollars;

    vec![
        GroupKpiView {
            label: "Groups",
            value: real.len().to_string(),
            note: format!("{unassigned} in no group"),
            tone: "",
        },
        GroupKpiView {
            label: "People in a group",
            value: members.to_string(),
            note: format!(
                "{} active",
                rows.iter().map(|r| r.active_members_30d).sum::<i64>()
            ),
            tone: "",
        },
        GroupKpiView {
            label: "Requests",
            value: requests.to_string(),
            note: format!("last {range_label}"),
            tone: "",
        },
        GroupKpiView {
            label: "Cost",
            value: format_usd(cost),
            note: "exclusive attribution".to_owned(),
            tone: "accent",
        },
        GroupKpiView {
            label: "Unattributed",
            value: format!("{}%", unattributed.share_pct),
            note: format!("{} requests", unattributed.requests),
            tone: if unattributed.share_pct > 25 {
                "warn"
            } else {
                "ok"
            },
        },
        GroupKpiView {
            label: "Missing attribution keys",
            value: unkeyed_people.to_string(),
            note: if unkeyed_people == 0 {
                "all accounts keyed".to_owned()
            } else {
                "spend lands unattributed".to_owned()
            },
            tone: if unkeyed_people == 0 { "ok" } else { "warn" },
        },
    ]
}

#[expect(
    clippy::too_many_arguments,
    reason = "page query plumbing; splitting the parameters is tracked in docs/tech-debt.md"
)]
pub(super) fn paginate(
    rows: Vec<GroupRowView>,
    page: i64,
    range: &str,
    sort: &str,
    dir: &str,
    source: &str,
) -> (Vec<GroupRowView>, Pagination) {
    let total_rows = rows.len() as i64;
    // Why: `i64::div_ceil` is unstable on the pinned toolchain, so the ceiling
    // is spelled out as the quotient plus a partial page. Written this way
    // rather than as `(n + size - 1) / size` because that form reads as a
    // rounding trick and clippy asks for the intrinsic that is not available.
    let total_pages = (total_rows / PAGE_SIZE + i64::from(total_rows % PAGE_SIZE != 0)).max(1);
    let current_page = page.clamp(1, total_pages);
    let start = (current_page - 1) * PAGE_SIZE;
    let window: Vec<GroupRowView> = rows
        .into_iter()
        .skip(start.max(0) as usize)
        .take(PAGE_SIZE as usize)
        .collect();
    let last_row = start + window.len() as i64;
    let source_query = super::sorting::source_query(source);
    let url =
        |p: i64| format!("{BASE_URL}?range={range}&sort={sort}&dir={dir}{source_query}&page={p}");

    let pagination = Pagination {
        current_page,
        total_pages,
        first_row: if window.is_empty() { 0 } else { start + 1 },
        last_row,
        total_rows,
        noun: "groups",
        has_prev: current_page > 1,
        has_next: current_page < total_pages,
        prev_url: (current_page > 1).then(|| url(current_page - 1)),
        next_url: (current_page < total_pages).then(|| url(current_page + 1)),
    };
    (window, pagination)
}
