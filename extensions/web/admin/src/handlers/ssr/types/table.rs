//! The pieces every admin table renders the same way.
//!
//! A sortable column header and a filter chip look identical on every listing
//! because they are the same control, so they are declared once here and the
//! pages fill them in. Seven pages had each written their own copy of the
//! sort header before this, which is how a column ends up sorting on one page
//! and not on another.
//!
//! The other two shared table pieces live in
//! [`crate::handlers::ssr::list_view`] rather than here, because they are
//! built by functions in that module: `Pagination`, which the
//! `components/pagination` partial reads, and `Chip`, the removable chip the
//! identity filter ribbon renders for a filter already in force.

use serde::Serialize;

/// One sortable column header, as `components/sort-header` reads it.
#[derive(Debug, Clone, Serialize)]
pub struct SortHeaderView {
    pub label: &'static str,
    pub class: &'static str,
    // Why: the column explanation lives on the `th` because the row-wide link
    // overlay would cover the same tooltip on a cell.
    pub hint: &'static str,
    pub url: String,
    pub active: bool,
    pub aria_sort: &'static str,
    pub indicator: &'static str,
}

// Why: One column an operator can order a list by. Four pages declared this
// same `{key, label, class, hint}` under three different names, which is why
// `sort_headers` could not be shared across them.
#[derive(Debug, Clone, Copy)]
pub struct SortColumn {
    pub key: &'static str,
    pub label: &'static str,
    pub class: &'static str,
    pub hint: &'static str,
}

impl SortHeaderView {
    // Why: an active column toggles; an inactive one opens descending, because
    // every numeric column on these pages is scanned for its largest values
    // and every date column for its most recent. A page needs this before it
    // can build the link, which is why it is separate from `new`.
    #[must_use]
    pub const fn next_dir(active: bool, descending: bool) -> &'static str {
        if active && descending { "asc" } else { "desc" }
    }

    // Why: twelve pages each derived `aria_sort` and `indicator` from the same
    // two booleans, and `aria_sort` was spelled three different ways among
    // them — an accessibility semantic settled by whichever page you landed
    // on. The URL is the only part a page genuinely owns, so it is the only
    // part it passes.
    #[must_use]
    pub const fn new(
        (label, class, hint): (&'static str, &'static str, &'static str),
        url: String,
        active: bool,
        descending: bool,
    ) -> Self {
        Self {
            label,
            class,
            hint,
            url,
            active,
            aria_sort: if !active {
                "none"
            } else if descending {
                "descending"
            } else {
                "ascending"
            },
            indicator: if !active {
                "\u{2195}"
            } else if descending {
                "\u{25bc}"
            } else {
                "\u{25b2}"
            },
        }
    }
}

// Why: One quick filter above a listing: a labelled link that narrows the rows,
// carrying the count it would leave behind.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct FilterChipView {
    pub label: &'static str,
    pub href: String,
    pub is_active: bool,
    pub count: i64,
}
