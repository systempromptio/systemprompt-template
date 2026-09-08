//! One sortable-column builder for every platform list page.
//!
//! The catalog lists are small enough to sort in memory — they come from YAML
//! on disk, not from a query — so what varies between them is only the column
//! set and the comparator. This module owns the URL and ARIA half of that, so
//! adding a column to a page is naming it, not re-deriving the link, the
//! indicator glyph and the `aria-sort` value a fourth time.

// Why: re-exported rather than merely imported — this module is the catalog's
// sort-header builder, and `lib.rs` and the catalog views name the type
// through it.
pub(crate) use crate::handlers::ssr::types::SortHeaderView;

// Why: One column an operator can order a platform list by.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SortColumn {
    pub key: &'static str,
    pub label: &'static str,
    pub class: &'static str,
    pub hint: &'static str,
}

// Why: The header row for `columns`, given the base path and the applied order.
//
// An inactive column opens descending, because every numeric column on these
// pages is scanned for its largest values and every date column for its most
// recent; an active one flips. `preserved` is the rest of the query string —
// a sort link that dropped the active filter would silently widen the list
// the operator is looking at.
pub(crate) fn sort_headers(
    base_url: &str,
    columns: &[SortColumn],
    active_key: &str,
    active_dir: &str,
    preserved: &str,
) -> Vec<SortHeaderView> {
    let prefix = if preserved.is_empty() {
        format!("{base_url}?")
    } else {
        format!("{base_url}?{preserved}&")
    };
    columns
        .iter()
        .map(|col| {
            let active = col.key == active_key;
            let next_dir = if active && active_dir == "desc" {
                "asc"
            } else {
                "desc"
            };
            SortHeaderView {
                label: col.label,
                class: col.class,
                hint: col.hint,
                url: format!("{prefix}sort={}&dir={next_dir}", col.key),
                active,
                aria_sort: if !active {
                    "none"
                } else if active_dir == "asc" {
                    "ascending"
                } else {
                    "descending"
                },
                indicator: if !active {
                    "\u{2195}"
                } else if active_dir == "asc" {
                    "\u{25b2}"
                } else {
                    "\u{25bc}"
                },
            }
        })
        .collect()
}

// Why: Normalise `?dir=` to the two values the headers understand.
pub(crate) fn direction(raw: Option<&str>) -> &'static str {
    if raw == Some("asc") { "asc" } else { "desc" }
}

// Why: The `q=` filter, url-encoded for a link, or an empty string.
pub(crate) fn preserved_search(q: &str) -> String {
    if q.is_empty() {
        return String::new();
    }
    format!("q={}", urlencoding::encode(q))
}

// Why: Case-insensitive substring match over a row's searchable text.
pub(crate) fn matches(haystack: &[&str], needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    let needle = needle.to_lowercase();
    haystack
        .iter()
        .any(|field| field.to_lowercase().contains(&needle))
}

// Why: Apply `dir` to a comparator that always orders ascending.
pub(crate) fn apply_direction<T>(
    rows: &mut [T],
    dir: &str,
    cmp: impl Fn(&T, &T) -> std::cmp::Ordering,
) {
    rows.sort_by(|a, b| {
        let ord = cmp(a, b);
        if dir == "asc" { ord } else { ord.reverse() }
    });
}
