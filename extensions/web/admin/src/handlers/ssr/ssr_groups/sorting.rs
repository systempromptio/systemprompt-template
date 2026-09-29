//! Column ordering and the window picker for the groups listing.
//!
//! The listing is bounded by the number of groups an estate has, so it is
//! ordered in memory rather than in SQL: the spend columns arrive from the
//! totals query, which partitions the instance and therefore cannot be
//! narrowed to one page without breaking the property that makes it worth
//! reading.

use std::cmp::Ordering;

use super::super::types::{FilterLinkView, GroupRowView, GroupSortHeaders, SortHeaderView};
use crate::handlers::ssr::types::table::SortColumn;

pub(super) const BASE_URL: &str = "/admin/groups";

// Why: the window every figure on the page is measured over. Ninety days is
// offered because a quarter is the unit a spend conversation happens in.
pub(super) const RANGES: [(&str, &str, i32); 3] = [
    ("7d", "7 days", 7),
    ("30d", "30 days", 30),
    ("90d", "90 days", 90),
];

const COLUMNS: [SortColumn; 9] = [
    SortColumn {
        key: "name",
        label: "Group",
        class: "",
        hint: "The group's display name",
    },
    SortColumn {
        key: "source",
        label: "Source",
        class: "",
        hint: "Who created the group: the directory loader, the dashboard, or the system",
    },
    SortColumn {
        key: "members",
        label: "Members",
        class: "sp-table__cell--num",
        hint: "Everyone in the group, and how many made a request in the window",
    },
    SortColumn {
        key: "active",
        label: "Active",
        class: "sp-table__cell--num",
        hint: "Members who made a request in the window",
    },
    SortColumn {
        key: "projects",
        label: "Projects",
        class: "sp-table__cell--num",
        hint: "Distinct projects this group's members work on",
    },
    SortColumn {
        key: "model",
        label: "Top model",
        class: "",
        hint: "The model this group sent the most requests to",
    },
    SortColumn {
        key: "requests",
        label: "Requests",
        class: "sp-table__cell--num",
        hint: "Requests attributed exclusively to this group",
    },
    SortColumn {
        key: "tokens",
        label: "Tokens",
        class: "sp-table__cell--num",
        hint: "Input plus output tokens, exclusively attributed",
    },
    SortColumn {
        key: "cost",
        label: "Cost",
        class: "sp-table__cell--num",
        hint: "Spend attributed exclusively to this group",
    },
];

pub(super) fn direction(raw: Option<&str>) -> &'static str {
    if raw == Some("asc") { "asc" } else { "desc" }
}

pub(super) fn sort_key(raw: Option<&str>) -> &'static str {
    COLUMNS
        .iter()
        .find(|c| Some(c.key) == raw)
        .map_or("cost", |c| c.key)
}

pub(super) fn window_days(raw: Option<&str>) -> i32 {
    RANGES
        .iter()
        .find(|(value, _, _)| Some(*value) == raw)
        .map_or(30, |(_, _, days)| *days)
}

pub(super) fn range_label(days: i32) -> &'static str {
    RANGES
        .iter()
        .find(|(_, _, d)| *d == days)
        .map_or("30 days", |(_, label, _)| *label)
}

pub(super) fn range_links(days: i32, source: &str) -> Vec<FilterLinkView> {
    RANGES
        .iter()
        .map(|(value, label, d)| FilterLinkView {
            label,
            value,
            url: format!("{BASE_URL}?range={value}{}", source_query(source)),
            active: *d == days,
        })
        .collect()
}

// Why: the source filter is the page's second dimension — who created the
// group. It is a link like every other control here, so a filtered view is a
// URL somebody can send.
pub(super) fn source_query(source: &str) -> String {
    if source.is_empty() {
        String::new()
    } else {
        format!("&source={source}")
    }
}

pub(super) fn source_links(selected: &str, range: &str) -> Vec<FilterLinkView> {
    [
        ("", "All sources"),
        ("Directory", "Directory"),
        ("Dashboard", "Created here"),
        ("System", "Built-in"),
    ]
    .into_iter()
    .map(|(value, label)| FilterLinkView {
        label,
        value,
        url: format!("{BASE_URL}?range={range}{}", source_query(value)),
        active: value == selected,
    })
    .collect()
}

// Why: only the four labels the listing renders are accepted, so a hand-typed
// `?source=` narrows to nothing rather than being ignored — a filter that
// silently widens is worse than one that shows an empty table.
pub(super) fn source_filter(raw: Option<&str>) -> String {
    match raw {
        Some(v @ ("Directory" | "Dashboard" | "System")) => v.to_owned(),
        _ => String::new(),
    }
}

// Why: an inactive column opens descending. Every numeric column here is
// scanned for its largest values, and the name column is the only one anyone
// reads upward — which is what `dir=asc` on a second click is for.
pub(super) fn sort_headers(key: &str, dir: &str, range: &str, source: &str) -> GroupSortHeaders {
    // Why: destructuring the fixed-size array binds each column by position,
    // so a column added to COLUMNS without a struct field (or vice versa) is a
    // compile error rather than an empty `th`.
    let [
        name,
        source,
        members,
        active,
        projects,
        model,
        requests,
        tokens,
        cost,
    ] = COLUMNS
        .each_ref()
        .map(|col| header(col, key, dir, range, source));
    GroupSortHeaders {
        name,
        source,
        members,
        active,
        projects,
        model,
        requests,
        tokens,
        cost,
    }
}

fn header(col: &SortColumn, key: &str, dir: &str, range: &str, source: &str) -> SortHeaderView {
    let active = col.key == key;
    let descending = dir != "asc";
    let next = SortHeaderView::next_dir(active, descending);
    SortHeaderView::new(
        (col.label, col.class, col.hint),
        format!(
            "{BASE_URL}?range={range}&sort={}&dir={next}{}",
            col.key,
            source_query(source)
        ),
        active,
        descending,
    )
}

// Why: the derived Unassigned bucket sorts to the end whatever the column,
// because it is not a group anyone created and letting it win a spend sort
// would put a bucket at the top of a list of teams.
pub(super) fn apply(rows: &mut [GroupRowView], key: &str, dir: &str) {
    rows.sort_by(|a, b| {
        a.is_unassigned
            .cmp(&b.is_unassigned)
            .then_with(|| order(a, b, key, dir))
    });
}

fn order(a: &GroupRowView, b: &GroupRowView, key: &str, dir: &str) -> Ordering {
    let ord = match key {
        "name" => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        "source" => a.source.cmp(&b.source),
        "members" => a.member_count.cmp(&b.member_count),
        "active" => a.active_members_30d.cmp(&b.active_members_30d),
        "projects" => a.project_count.cmp(&b.project_count),
        "model" => a.top_model.cmp(&b.top_model),
        "requests" => a.requests.cmp(&b.requests),
        "tokens" => a.tokens.cmp(&b.tokens),
        _ => a.cost_microdollars.cmp(&b.cost_microdollars),
    };
    let ord = ord.then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    if dir == "asc" { ord } else { ord.reverse() }
}
