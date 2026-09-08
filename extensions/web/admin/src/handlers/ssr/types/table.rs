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
pub(crate) struct SortHeaderView {
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

// Why: One quick filter above a listing: a labelled link that narrows the rows,
// carrying the count it would leave behind.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct FilterChipView {
    pub label: &'static str,
    pub href: String,
    pub is_active: bool,
    pub count: i64,
}
