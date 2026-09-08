//! The groups listing's own context types: one row, the remainder that
//! completes it, and the controls above it.
//!
//! They live apart from the detail page's types because the listing is where
//! the exclusive-attribution contract is visible — the rows plus
//! [`UnattributedRowView`] reproduce the instance total — and that claim is
//! easier to keep true when the shapes carrying it sit together.

use serde::Serialize;

use super::super::list_view::Pagination;
use super::{BreadcrumbView, MemberSetChipView, SortHeaderView};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct GroupRowView {
    pub id: String,
    pub name: String,
    pub href: String,
    pub description: Option<String>,
    pub member_count: i64,
    pub active_members_30d: i64,
    // Why: the chips are clipped to one line so the row stays 32px, so the
    // full list has to live somewhere the mouse can reach it.
    pub marketplaces_title: String,
    pub marketplace_count: i64,
    pub project_count: i64,
    pub requests: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
    // Why: the busiest model a group's own traffic used, which is the one
    // question a cost column always provokes and would otherwise cost a click.
    pub top_model: Option<String>,
    // Why: the vendor prefix repeats on every row of a single-vendor estate,
    // so the cell shows the tail and keeps the full id on its title.
    pub top_model_short: Option<String>,
    pub top_model_requests: i64,
    pub source: String,
    pub source_label: &'static str,
    pub source_abbrev: &'static str,
    pub is_unassigned: bool,
}

// Why: The remainder row. Under exclusive attribution every group's traffic
// plus this equals the instance, so it is a row of the same table rather than
// a footnote: dropping it would leave a listing that silently fails to add up.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct UnattributedRowView {
    pub requests: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
    pub share_pct: i64,
    pub people: i64,
}

// Why: One header tile. Named for the page rather than shared, because two
// other pages already define a `KpiView` of their own and the duplicate-type
// gate reads names, not shapes.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct GroupKpiView {
    pub label: &'static str,
    pub value: String,
    // Why: `note`, not `sub`. `sub` is a registered Handlebars helper, so
    // `this.sub` in the template resolves to the helper rather than the field
    // and the supporting line renders as nothing at all.
    pub note: String,
    pub tone: &'static str,
}

// Why: One entry of a link-shaped filter: the time window and the source
// picker both render as these.
//
// It carries `value` as well as a label because the control's machine-readable
// identity is what the `data-scope-range` and `data-source-filter` hooks bind,
// and that is what distinguishes it from `FilterChipView`, which carries a
// count instead.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct FilterLinkView {
    pub label: &'static str,
    pub value: &'static str,
    pub url: String,
    pub active: bool,
}

// Why: the header set is named rather than a positional list. Two columns of
// the listing are not sortable and sit between ones that are, so a template
// reading a Vec has to index around them — and every index shifts the day a
// column is added. Naming them makes the template say which column it is
// drawing, and makes a missing one a compile error rather than a blank cell.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct GroupSortHeaders {
    pub name: SortHeaderView,
    pub source: SortHeaderView,
    pub members: SortHeaderView,
    pub active: SortHeaderView,
    pub projects: SortHeaderView,
    pub model: SortHeaderView,
    pub requests: SortHeaderView,
    pub tokens: SortHeaderView,
    pub cost: SortHeaderView,
}

#[derive(Debug, Serialize)]
pub(crate) struct GroupsPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub kpis: Vec<GroupKpiView>,
    pub sort_headers: GroupSortHeaders,
    pub ranges: Vec<FilterLinkView>,
    pub sources: Vec<FilterLinkView>,
    pub source_filter: String,
    pub range_label: &'static str,
    pub toolbar_count: String,
    pub window_days: i32,
    pub groups: Vec<GroupRowView>,
    pub unattributed: UnattributedRowView,
    pub pagination: Pagination,
    pub total_groups: i64,
    pub can_manage: bool,
    pub can_map: bool,
    // Why: destinations for the header's "Map a directory group" dialog, which
    // is where a mapping is created from — the group it lands in is a field of
    // the form rather than a page you must first navigate to.
    pub group_options: Vec<MemberSetChipView>,
    // Why: people with no primary group at all. Exclusive attribution has
    // nowhere to file their spend, so the page says how many there are and
    // offers the recompute rather than quietly reporting a smaller instance.
    pub unkeyed_people: i64,
}
