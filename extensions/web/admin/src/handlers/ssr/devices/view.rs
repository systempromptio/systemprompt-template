//! Turning fleet rows into the page's tiles, chips, headers and links.
//!
//! Every URL this module builds carries the whole view state — tab, filter,
//! sort and page — because each of them is a link rather than a script. A tab
//! that dropped the filter would silently widen what the reader is looking at,
//! so the query string is rebuilt from the parsed state in one place instead
//! of being patched per control.

use crate::handlers::ssr::list_view::{PageWindow, Pagination};
use crate::handlers::ssr::types::{FilterChipView, SortHeaderView, TabLinkView};
use crate::repositories::devices::sessions::VersionCount;
use crate::repositories::devices::stats::FleetStats;

use super::context::{FleetStatsView, VersionBarView};
use super::{BASE_URL, DevicesQuery, Tab};

pub(super) fn url_with(query: &DevicesQuery, overrides: &[(&str, &str)]) -> String {
    let mut parts: Vec<(&str, String)> = Vec::new();
    let base: [(&str, Option<&str>); 5] = [
        ("tab", query.tab.as_deref()),
        ("state", query.state.as_deref()),
        ("sort", query.sort.as_deref()),
        ("dir", query.dir.as_deref()),
        ("stale", query.stale.as_deref()),
    ];
    for (name, value) in base {
        if overrides.iter().any(|(k, _)| *k == name) {
            continue;
        }
        if let Some(value) = value.filter(|v| !v.is_empty()) {
            parts.push((name, value.to_owned()));
        }
    }
    for (name, value) in overrides {
        if !value.is_empty() {
            parts.push((name, (*value).to_owned()));
        }
    }
    if parts.is_empty() {
        return BASE_URL.to_owned();
    }
    let qs = parts
        .iter()
        .map(|(k, v)| format!("{k}={}", urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");
    format!("{BASE_URL}?{qs}")
}

pub(super) fn build_tabs(
    query: &DevicesQuery,
    active: Tab,
    stats: &FleetStats,
) -> Vec<TabLinkView> {
    let rows: [(Tab, &'static str, &'static str, i64); 4] = [
        (
            Tab::Bridges,
            "bridges",
            "Bridge sessions",
            stats.bridges_total,
        ),
        (Tab::Pats, "pats", "Access tokens", stats.pats_total),
        (
            Tab::Certs,
            "certs",
            "Device certificates",
            stats.certs_total,
        ),
        (Tab::Links, "links", "Pending links", stats.links_pending),
    ];
    rows.into_iter()
        .map(|(tab, slug, label, count)| TabLinkView {
            slug,
            label,
            // Why: switching tab drops the sort and filter with it. They name
            // columns and states that only exist on the tab that set them,
            // and carrying them across would sort by a column that is not there.
            href: url_with(
                query,
                &[
                    ("tab", slug),
                    ("sort", ""),
                    ("dir", ""),
                    ("state", ""),
                    ("stale", ""),
                ],
            ),
            is_active: tab == active,
            count: Some(count),
        })
        .collect()
}

pub(super) fn build_stats(query: &DevicesQuery, stats: &FleetStats) -> FleetStatsView {
    let stale_active = query.stale.as_deref() == Some("1");
    FleetStatsView {
        bridges_active: stats.bridges_active.to_string(),
        bridges_stale: stats.bridges_stale.to_string(),
        bridges_total_sub: format!("of {} enrolled", stats.bridges_total),
        stale_tone: if stats.bridges_stale > 0 {
            "warn"
        } else {
            "ok"
        },
        stale_url: url_with(
            query,
            &[
                ("tab", "bridges"),
                ("stale", if stale_active { "" } else { "1" }),
            ],
        ),
        stale_active,
        versions: stats.versions.to_string(),
        versions_sub: "distinct builds".to_owned(),
        pats_active: stats.pats_active.to_string(),
        pats_sub: format!("of {} issued", stats.pats_total),
        certs_active: stats.certs_active.to_string(),
        certs_sub: format!("of {} enrolled", stats.certs_total),
        links_pending: stats.links_pending.to_string(),
        links_sub: format!("{} expired", stats.links_expired),
        links_tone: if stats.links_expired > 0 {
            "warn"
        } else {
            "accent"
        },
    }
}

pub(super) fn build_version_bars(rows: &[VersionCount]) -> Vec<VersionBarView> {
    let max = rows.iter().map(|r| r.devices).max().unwrap_or(0);
    // Why: a string max ranks "0.9.0" above "0.47.0". Compare the dotted
    // components numerically; anything unparsable ranks below every real version.
    let latest = rows
        .iter()
        .filter_map(|r| {
            semver::Version::parse(&r.bridge_version)
                .ok()
                .map(|version| (version, r.bridge_version.as_str()))
        })
        .max_by(|(a, _), (b, _)| a.cmp(b))
        .map(|(_, label)| label);
    rows.iter()
        .map(|r| VersionBarView {
            pct: if max > 0 { r.devices * 100 / max } else { 0 },
            is_latest: Some(r.bridge_version.as_str()) == latest,
            label: r.bridge_version.clone(),
            devices: r.devices,
        })
        .collect()
}

// Why: the counts come from the fleet totals already read for the KPI strip
// rather than from three more aggregates. A chip states what it would leave
// behind, and the number it states is the one the tile above it states.
pub(super) fn build_state_chips(
    query: &DevicesQuery,
    active: &str,
    totals: (i64, i64),
) -> Vec<FilterChipView> {
    let (all, live) = totals;
    [
        ("All", "all", all),
        ("Active", "active", live),
        ("Revoked", "revoked", all - live),
    ]
    .into_iter()
    .map(|(label, value, count)| FilterChipView {
        label,
        href: url_with(query, &[("state", value)]),
        is_active: active == value,
        count,
    })
    .collect()
}

pub(super) fn build_stale_chips(query: &DevicesQuery, stats: &FleetStats) -> Vec<FilterChipView> {
    let stale_on = query.stale.as_deref() == Some("1");
    vec![
        FilterChipView {
            label: "All bridges",
            href: url_with(query, &[("stale", "")]),
            is_active: !stale_on,
            count: stats.bridges_total,
        },
        FilterChipView {
            label: "Stale over 7 days",
            href: url_with(query, &[("stale", "1")]),
            is_active: stale_on,
            count: stats.bridges_stale,
        },
    ]
}

pub(super) struct ColumnSpec {
    pub(super) key: &'static str,
    pub(super) label: &'static str,
    pub(super) class: &'static str,
    pub(super) hint: &'static str,
}

pub(super) fn build_sort_headers(
    query: &DevicesQuery,
    columns: &[ColumnSpec],
    active_sort: &str,
    active_dir: &str,
) -> Vec<SortHeaderView> {
    columns
        .iter()
        .map(|column| {
            let active = column.key == active_sort;
            let next_dir = if active && active_dir == "desc" {
                "asc"
            } else {
                "desc"
            };
            SortHeaderView {
                label: column.label,
                class: column.class,
                hint: column.hint,
                url: url_with(query, &[("sort", column.key), ("dir", next_dir)]),
                active,
                aria_sort: match (active, active_dir) {
                    (false, _) => "none",
                    (true, "asc") => "ascending",
                    (true, _) => "descending",
                },
                indicator: match (active, active_dir) {
                    (false, _) => "\u{2195}",
                    (true, "asc") => "\u{2191}",
                    (true, _) => "\u{2193}",
                },
            }
        })
        .collect()
}

pub(super) fn build_pagination(query: &DevicesQuery, window: PageWindow) -> Pagination {
    let page = window.index;
    let prev_url = (page > 0).then(|| url_with(query, &[("page", &(page - 1).to_string())]));
    let next_url = (page + 1 < window.total_pages)
        .then(|| url_with(query, &[("page", &(page + 1).to_string())]));
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
