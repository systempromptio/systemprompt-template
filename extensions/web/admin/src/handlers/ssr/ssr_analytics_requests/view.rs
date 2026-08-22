//! View-model assembly for the Inference Requests page.
//!
//! Pure functions that turn repository rows + the parsed query into the typed
//! context the `analytics-requests` template consumes: KPI strip, the two
//! charts, the breakdown tables, paged rows, the tab bar, and the URL builders
//! that preserve query state across tabs, pagination, and the time presets.

use crate::handlers::ssr::format::{format_cost, format_duration_ms};
use crate::handlers::ssr::types::bar_pct;
use crate::repositories::analytics::request_stats::RequestStats;
use crate::repositories::analytics::requests::{
    BreakdownRow, RequestFilter, RequestRow, RequestSortColumn, RequestSortSpec, SortDir,
};
use crate::util::org_scope::OrgScope;
use crate::util::time_range::TimeRange;

use super::context::{
    BreakdownRowView, BreakdownView, RequestListRowView, RequestStatsView, RequestsTab,
    TimeRangeView,
};
use super::urls::{log_filter_url, preserved_query_string};
use super::{BASE_URL, RequestsQuery};

// Why: `org_scope` is the caller's resolved organization. Only a platform
// admin arrives unpinned, and `?org=` is then theirs to narrow with; pinning
// everyone else to their own slug is what keeps a customer's own admin inside
// their tenant.
pub(super) fn filter_from_query(query: &RequestsQuery, org_scope: OrgScope) -> RequestFilter {
    let org_slug = match org_scope {
        OrgScope::AllOrganizations => {
            empty_to_none(query.org.as_ref()).map_or(OrgScope::AllOrganizations, OrgScope::Only)
        },
        own @ OrgScope::Only(_) => own,
    };
    RequestFilter {
        org_slug,
        department: empty_to_none(query.department.as_ref()),
        user_id: query.user_id.clone().filter(|u| !u.as_str().is_empty()),
        agent_id: query.agent_id.clone().filter(|a| !a.as_str().is_empty()),
        model: empty_to_none(query.model.as_ref()),
        provider: empty_to_none(query.provider.as_ref()),
        status: empty_to_none(query.status.as_ref()),
        search: empty_to_none(query.q.as_ref()),
    }
}

fn empty_to_none(v: Option<&String>) -> Option<String> {
    v.map(String::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

pub(super) fn sort_from_query(query: &RequestsQuery) -> RequestSortSpec {
    let column = match query.sort.as_deref() {
        Some("cost") => RequestSortColumn::Cost,
        Some("latency") => RequestSortColumn::Latency,
        Some("tokens") => RequestSortColumn::Tokens,
        _ => RequestSortColumn::CreatedAt,
    };
    let dir = match query.dir.as_deref() {
        Some("asc") => SortDir::Asc,
        _ => SortDir::Desc,
    };
    RequestSortSpec { column, dir }
}

pub(super) fn stats_to_json(s: &RequestStats) -> RequestStatsView {
    RequestStatsView {
        total: s.total,
        error_count: s.error_count,
        requests_per_minute: format!("{:.2}", s.requests_per_minute),
        p50_latency_ms: s.p50_latency_ms.round() as i64,
        p95_latency_ms: s.p95_latency_ms.round() as i64,
        p99_latency_ms: s.p99_latency_ms.round() as i64,
        total_cost_display: format_cost(s.total_cost_microdollars),
        error_rate_pct: format!("{:.2}", s.error_rate * 100.0),
        denied_session_count: s.denied_session_count,
        denied_session_rate_pct: format!("{:.2}", s.denied_session_rate * 100.0),
    }
}

// Why: share_pct is against the busiest row rather than the window total, so
// the bars use the full width even when one dimension has a long tail. The
// printed percentage is still share-of-total.
pub(super) fn breakdown_view(
    tab: RequestsTab,
    rows: &[BreakdownRow],
    query: &RequestsQuery,
) -> BreakdownView {
    let (dimension_label, caption, param) = match tab {
        RequestsTab::Providers => (
            "Provider",
            "Traffic, spend, and failures rolled up to the upstream provider.",
            "provider",
        ),
        RequestsTab::Status => (
            "Status",
            "Outcome mix for the window. Failed calls still bill for the tokens they consumed.",
            "status",
        ),
        _ => (
            "Model",
            "Traffic, spend, and failures attributed to the model that produced them.",
            "model",
        ),
    };

    let max = rows.iter().map(|r| r.requests).max().unwrap_or(0);
    let total: i64 = rows.iter().map(|r| r.requests).sum();

    BreakdownView {
        dimension_label,
        caption,
        has_rows: !rows.is_empty(),
        rows: rows
            .iter()
            .map(|r| BreakdownRowView {
                requests: r.requests,
                share_pct: bar_pct(r.requests, max),
                share_display: format!("{:.1}%", pct_of(r.requests, total)),
                tokens_display: format!(
                    "{} / {}",
                    compact_int(r.input_tokens),
                    compact_int(r.output_tokens)
                ),
                cost_display: format_cost(r.cost_microdollars),
                p50_display: format_duration_ms(r.p50_latency_ms.round() as i64),
                p95_display: format_duration_ms(r.p95_latency_ms.round() as i64),
                error_count: r.error_count,
                error_rate_display: format!("{:.1}%", pct_of(r.error_count, r.requests)),
                has_errors: r.error_count > 0,
                filter_url: log_filter_url(query, param, &r.key),
                key: r.key.clone(),
            })
            .collect(),
    }
}

fn pct_of(part: i64, whole: i64) -> f64 {
    if whole <= 0 {
        return 0.0;
    }
    part as f64 / whole as f64 * 100.0
}

fn compact_int(v: i64) -> String {
    if v >= 1_000_000 {
        format!("{:.1}M", v as f64 / 1_000_000.0)
    } else if v >= 10_000 {
        format!("{}k", v / 1000)
    } else if v >= 1000 {
        format!("{:.1}k", v as f64 / 1000.0)
    } else {
        v.to_string()
    }
}

pub(super) fn request_row_to_json(r: &RequestRow) -> RequestListRowView {
    RequestListRowView {
        id: r.id.clone(),
        request_id: r.request_id.clone(),
        trace_id: r.trace_id.clone(),
        session_id: r.session_id.clone(),
        user_id: r.user_id.clone(),
        user_label: r
            .user_label
            .clone()
            .unwrap_or_else(|| r.user_id.as_str().to_owned()),
        provider: r.provider.clone(),
        model: r.model.clone(),
        status: r.status.clone(),
        is_error: is_error_status(&r.status),
        input_tokens: r.input_tokens,
        output_tokens: r.output_tokens,
        tokens_total: r.input_tokens.unwrap_or(0) + r.output_tokens.unwrap_or(0),
        cost_microdollars: r.cost_microdollars,
        cost_display: format_cost(r.cost_microdollars),
        latency_ms: r.latency_ms,
        error_message: r.error_message.clone(),
        decision_count: r.decision_count,
        deny_count: r.deny_count,
        is_denied_preflight: r.deny_count > 0,
        tool_call_count: r.tool_call_count,
        created_at: r.created_at.to_rfc3339(),
        created_at_local: r
            .created_at
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
    }
}

fn is_error_status(status: &str) -> bool {
    !matches!(status, "completed" | "pending" | "streaming")
}

pub(super) fn time_range_context(
    query: &RequestsQuery,
    range: &TimeRange,
    auto_widened: Option<&'static str>,
) -> TimeRangeView {
    let preset = query.preset.clone().unwrap_or_else(|| {
        if query.from.is_some() && query.to.is_some() {
            "custom".to_owned()
        } else {
            auto_widened.unwrap_or("24h").to_owned()
        }
    });
    let qs = preserved_query_string(query, &["preset", "from", "to"]);
    let q_suffix = if qs.is_empty() {
        String::new()
    } else {
        format!("&{qs}")
    };
    TimeRangeView {
        preset,
        from: range.from.to_rfc3339(),
        to: range.to.to_rfc3339(),
        base_url: BASE_URL,
        query: q_suffix,
        auto_widened,
    }
}
