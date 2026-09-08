//! The pieces every filtered entity-list page renders the same way.
//!
//! Every filtered list page binds to the same URL contract —
//! `?preset=&from=&to=&<facet>=&page=` — and feeds the same
//! `components/time-range`, `components/identity-filter-ribbon` and pagination
//! partials. The types below are the serialisation those partials read, and
//! the functions build them from a page's own query parameters expressed as a
//! `(name, value)` slice, so a list page supplies its parameter list and
//! inherits the behaviour rather than copying it.

pub(crate) use systemprompt_web_shared::pagination::PageWindow;

use serde::Serialize;

use crate::util::time_range::{TimeRange, TimeRangePreset, TimeRangeQuery};

// Why: A page's query parameters, in the order they should appear in a rebuilt
// URL.

#[derive(Debug, Serialize)]
pub(crate) struct TimeRangeContext {
    pub(crate) preset: String,
    pub(crate) from: String,
    pub(crate) to: String,
    pub(crate) base_url: &'static str,
    pub(crate) query: &'static str,
    // Why: carried from `TimeRange::rejected_bounds` so the partial can say
    // the window is the default rather than the one the URL asked for. A
    // listing that answers for a different window without saying so is read as
    // the answer to the question that was asked.
    pub(crate) rejected: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SelectOptionView {
    pub value: String,
    pub label: String,
    pub selected: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct Preserved {
    pub(crate) name: &'static str,
    pub(crate) value: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct AnnotatedOption {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) count: i64,
    pub(crate) selected: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct Chip {
    pub(crate) group_label: &'static str,
    pub(crate) label: String,
    pub(crate) value: String,
    pub(crate) remove_url: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct Pagination {
    pub(crate) current_page: i64,
    pub(crate) total_pages: i64,
    // Why: 1-based row range for "Showing 1-50 of 54"; `first_row` is 0 only
    // when the page is empty.
    pub(crate) first_row: i64,
    pub(crate) last_row: i64,
    pub(crate) total_rows: i64,
    pub(crate) noun: &'static str,
    pub(crate) has_prev: bool,
    pub(crate) has_next: bool,
    pub(crate) prev_url: Option<String>,
    pub(crate) next_url: Option<String>,
}

// Why: Drops empty values so a rebuilt URL never carries `&user_id=`.
pub(crate) fn empty_to_none(v: Option<&str>) -> Option<&str> {
    v.filter(|s| !s.is_empty())
}

// Why: The preset name to echo back into the URL.
//
// An explicit `?preset=` wins, then an explicit `from`+`to` pair means
// `custom`; otherwise the parsed range's own preset is authoritative.
pub(crate) fn preset_str(query: &TimeRangeQuery, range: TimeRange) -> String {
    if let Some(p) = empty_to_none(query.preset.as_deref()) {
        return p.to_owned();
    }
    if query.from.is_some() && query.to.is_some() {
        return "custom".to_owned();
    }
    match range.preset {
        TimeRangePreset::Min15 => "15m",
        TimeRangePreset::Hour1 => "1h",
        TimeRangePreset::Hours24 => "24h",
        TimeRangePreset::Days7 => "7d",
        TimeRangePreset::Days30 => "30d",
        TimeRangePreset::Custom => "custom",
    }
    .to_owned()
}

pub(crate) fn time_range_context(
    base_url: &'static str,
    range: TimeRange,
    preset: &str,
) -> TimeRangeContext {
    TimeRangeContext {
        preset: preset.to_owned(),
        from: range.from.to_rfc3339(),
        to: range.to.to_rfc3339(),
        base_url,
        query: "",
        rejected: range.rejected_bounds,
    }
}

// Why: Hidden inputs the filter-ribbon form must resubmit so that choosing a
// facet does not silently reset the time window.
pub(crate) use super::list_scope::{
    HiddenFieldView, ScopeFilterView, scope_filter_from_names, scope_filter_view,
};
