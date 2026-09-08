//! Typed template-context structs for the Trace Explorer list page
//! (`perf-traces.hbs`) and the shared entity-view / time-range / identity
//! filter-ribbon partials it includes.

use serde::Serialize;

use crate::handlers::ssr::list_view::{
    AnnotatedOption, Chip, Pagination, Preserved, ScopeFilterView, TimeRangeContext,
};
use crate::handlers::ssr::types::{BreadcrumbView, SortHeaderView};

#[derive(Debug, Serialize)]
pub(super) struct PerfTracesPageContext {
    pub(super) page: &'static str,
    pub(super) title: &'static str,
    pub(super) breadcrumbs: Vec<BreadcrumbView>,
    pub(super) time_range: TimeRangeContext,
    pub(super) filter_ribbon: TraceFilterRibbon,
    pub(super) stats: TraceStatsView,
    pub(super) traces: Vec<super::rows::TraceRow>,
    pub(super) has_traces: bool,
    pub(super) total_count: i64,
    pub(super) page_size: i64,
    pub(super) page_index: i64,
    pub(super) page_count: i64,
    pub(super) pagination: Pagination,
    pub(super) sort_headers: TracesSortHeaders,
    pub(super) sort: &'static str,
    pub(super) dir: &'static str,
    pub(super) error_only: bool,
    pub(super) deny_only: bool,
    pub(super) scope_filter: ScopeFilterView,
}

#[derive(Debug, Serialize)]
pub(super) struct TraceFilterRibbon {
    pub(super) base_url: &'static str,
    pub(super) preserved: Vec<Preserved>,
    pub(super) options: TraceFilterOptionsView,
    pub(super) chips: Vec<Chip>,
}

#[derive(Debug, Default, Serialize)]
pub(super) struct TraceFilterOptionsView {
    pub(super) users: Vec<AnnotatedOption>,
    pub(super) agents: Vec<AnnotatedOption>,
    pub(super) agent_scopes: Vec<AnnotatedOption>,
    pub(super) policies: Vec<AnnotatedOption>,
    pub(super) decisions: Vec<AnnotatedOption>,
}

#[derive(Debug, Serialize)]
pub(super) struct TraceStatsView {
    pub(super) total_traces: i64,
    pub(super) error_count: i64,
    pub(super) deny_count: i64,
    pub(super) deny_url: String,
    pub(super) error_url: String,
    pub(super) deny_active: bool,
    pub(super) error_active: bool,
    pub(super) cost_display: String,
    pub(super) tokens_display: String,
    pub(super) p50_display: String,
    pub(super) p95_display: String,
    pub(super) p99_display: String,
}

// Why: named fields rather than a Vec so the template addresses each header by
// name and the column set is checked at compile time.
#[derive(Debug, Serialize)]
pub(super) struct TracesSortHeaders {
    pub(super) started: SortHeaderView,
    pub(super) activity: SortHeaderView,
    pub(super) tokens: SortHeaderView,
    pub(super) cost: SortHeaderView,
    pub(super) duration: SortHeaderView,
}
