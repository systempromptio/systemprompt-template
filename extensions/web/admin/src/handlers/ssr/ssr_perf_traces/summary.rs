//! Summary strip + sortable headers for the Trace Explorer list page.
//!
//! Split from `view.rs`, which keeps the filter ribbon, chips and pagination.
//! Both halves share `preserved_query_string` so every link on the page carries
//! the same filter and time-range state.

use crate::handlers::ssr::format::{format_cost, format_duration_ms, format_token_total};
use crate::repositories::traces::TraceStats;

use super::context::{SortHeader, SortHeaders, TraceStatsView};
use super::view::preserved_query_string;
use super::{BASE_URL, TraceListQuery};

pub(super) fn serde_stats(query: &TraceListQuery, s: &TraceStats) -> TraceStatsView {
    TraceStatsView {
        total_traces: s.total_traces,
        error_count: s.error_count,
        deny_count: s.deny_count,
        deny_url: toggle_flag_url(query, "deny_only"),
        error_url: toggle_flag_url(query, "error_only"),
        deny_active: query.deny_only.as_deref() == Some("true"),
        error_active: query.error_only.as_deref() == Some("true"),
        cost_display: format_cost(s.total_cost_microdollars),
        tokens_display: format_token_total(s.total_tokens),
        p50_display: format_duration_ms(s.p50_active_ms),
        p95_display: format_duration_ms(s.p95_active_ms),
        p99_display: format_duration_ms(s.p99_active_ms),
    }
}

fn toggle_flag_url(query: &TraceListQuery, flag: &str) -> String {
    let already_on = match flag {
        "deny_only" => query.deny_only.as_deref() == Some("true"),
        _ => query.error_only.as_deref() == Some("true"),
    };
    let qs = preserved_query_string(query, &[flag, "page"]);
    if already_on {
        return if qs.is_empty() {
            BASE_URL.to_owned()
        } else {
            format!("{BASE_URL}?{qs}")
        };
    }
    if qs.is_empty() {
        format!("{BASE_URL}?{flag}=true")
    } else {
        format!("{BASE_URL}?{qs}&{flag}=true")
    }
}

// Why: The five columns the list query can actually order by. Each header
// renders as a link that flips direction when it is already active, so the
// `cursor:pointer` the table CSS has always shown finally does something.
pub(super) fn build_sort_headers(
    query: &TraceListQuery,
    active_col: &str,
    active_dir: &str,
) -> SortHeaders {
    // Why: Every sort link carries the current filters and time range, minus the
    // sort state it is replacing and the page it would invalidate.
    let qs = preserved_query_string(query, &["sort", "dir", "page"]);
    let prefix = if qs.is_empty() {
        format!("{BASE_URL}?")
    } else {
        format!("{BASE_URL}?{qs}&")
    };
    let header = |key: &str, label: &'static str, class: &'static str, hint: &'static str| {
        let active = key == active_col;
        // Why: An active column toggles; an inactive one opens largest-first (and
        // newest-first for time), which is what an operator scans for.
        let next_dir = if active && active_dir == "desc" {
            "asc"
        } else {
            "desc"
        };
        SortHeader {
            label,
            class,
            hint,
            url: format!("{prefix}sort={key}&dir={next_dir}"),
            active,
            aria_sort: if active {
                if active_dir == "asc" {
                    "ascending"
                } else {
                    "descending"
                }
            } else {
                "none"
            },
            indicator: if active {
                if active_dir == "asc" { "▲" } else { "▼" }
            } else {
                "↕"
            },
        }
    };
    SortHeaders {
        started: header(
            "started_at",
            "Started",
            "col-started",
            "First event on the trace, in local time",
        ),
        activity: header(
            "spans",
            "Activity",
            "col-spans",
            "Gateway requests, then governance decisions and tool calls",
        ),
        tokens: header(
            "tokens",
            "Tokens",
            "col-tokens",
            "Total tokens, split input / output",
        ),
        cost: header("cost", "Cost", "col-cost", "Billed cost across the trace"),
        duration: header(
            "duration",
            "Duration",
            "col-duration",
            "Summed request latency, over the first-to-last event window",
        ),
    }
}
