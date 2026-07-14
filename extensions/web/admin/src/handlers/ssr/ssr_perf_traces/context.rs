//! Typed template-context structs for the Trace Explorer list page
//! (`perf-traces.hbs`) and the shared entity-view / time-range / identity
//! filter-ribbon partials it includes.

use serde::Serialize;

#[derive(Debug, Serialize)]
pub(super) struct PerfTracesPageContext {
    pub(super) page: &'static str,
    pub(super) title: &'static str,
    pub(super) entity_view_tabs: Vec<EntityViewTab>,
    pub(super) time_range: TimeRangeContext,
    pub(super) filter_ribbon: FilterRibbon,
    pub(super) stats: StatsView,
    pub(super) traces: Vec<super::rows::TraceRow>,
    pub(super) has_traces: bool,
    pub(super) total_count: i64,
    pub(super) page_size: i64,
    pub(super) page_index: i64,
    pub(super) page_count: i64,
    pub(super) pagination: Pagination,
    pub(super) sort: &'static str,
    pub(super) dir: &'static str,
    pub(super) error_only: bool,
    pub(super) deny_only: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct EntityViewTab {
    pub(super) key: &'static str,
    pub(super) label: &'static str,
    pub(super) url: String,
    pub(super) active: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct TimeRangeContext {
    pub(super) preset: String,
    pub(super) from: String,
    pub(super) to: String,
    pub(super) base_url: &'static str,
    pub(super) query: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct FilterRibbon {
    pub(super) base_url: &'static str,
    pub(super) preserved: Vec<Preserved>,
    pub(super) options: FilterOptionsView,
    pub(super) chips: Vec<Chip>,
}

#[derive(Debug, Serialize)]
pub(super) struct Preserved {
    pub(super) name: &'static str,
    pub(super) value: String,
}

#[derive(Debug, Default, Serialize)]
pub(super) struct FilterOptionsView {
    pub(super) users: Vec<AnnotatedOption>,
    pub(super) agents: Vec<AnnotatedOption>,
    pub(super) agent_scopes: Vec<AnnotatedOption>,
    pub(super) policies: Vec<AnnotatedOption>,
    pub(super) decisions: Vec<AnnotatedOption>,
}

#[derive(Debug, Serialize)]
pub(super) struct AnnotatedOption {
    pub(super) id: String,
    pub(super) label: String,
    pub(super) count: i64,
    pub(super) selected: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct Chip {
    pub(super) group_label: &'static str,
    pub(super) label: String,
    pub(super) value: String,
    pub(super) remove_url: String,
}

#[derive(Debug, Serialize)]
pub(super) struct StatsView {
    pub(super) total_traces: i64,
    pub(super) error_count: i64,
    pub(super) deny_count: i64,
    pub(super) p50_ms: i64,
    pub(super) p95_ms: i64,
    pub(super) p99_ms: i64,
}

#[derive(Debug, Serialize)]
pub(super) struct Pagination {
    pub(super) current_page: i64,
    pub(super) total_pages: i64,
    pub(super) has_prev: bool,
    pub(super) has_next: bool,
    pub(super) prev_url: Option<String>,
    pub(super) next_url: Option<String>,
}
