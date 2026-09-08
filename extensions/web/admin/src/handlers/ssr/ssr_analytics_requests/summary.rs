//! The numbers and pickers above the request log: the KPI tiles, the sortable
//! column headers, the filter selects, and the breakdown-tab rollups.
//!
//! Everything here is a pure function of what the repositories returned and
//! what the query asked for, so the page's arithmetic can be read without the
//! handler's I/O around it.

use crate::handlers::ssr::format::{format_cost, format_duration_ms};
use crate::handlers::ssr::list_view::{HiddenFieldView, SelectOptionView};
use crate::handlers::ssr::types::bar_pct;
use crate::repositories::analytics::request_stats::RequestStats;
use crate::repositories::analytics::requests::{BreakdownRow, FacetValue, RequestKpis};

use super::RequestsQuery;
use super::context::{
    BreakdownRowView, BreakdownView, FilterOptionsView, RequestKpiView, RequestStatsView,
    RequestsSortHeaders, RequestsTab,
};
use super::urls::{log_filter_url, sort_url_prefix};
use crate::handlers::ssr::types::SortHeaderView;

fn empty_to_none(v: Option<&String>) -> Option<String> {
    v.map(String::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

// Why: an active column toggles direction; an inactive one opens largest-first
// (newest-first for time), which is what an operator scans a log for.
pub(super) fn sort_headers(query: &RequestsQuery) -> RequestsSortHeaders {
    let active_col = query.sort.as_deref().unwrap_or("created_at");
    let active_dir = if query.dir.as_deref() == Some("asc") {
        "asc"
    } else {
        "desc"
    };
    let prefix = sort_url_prefix(query);
    let header = |key: &str, label: &'static str, class: &'static str, hint: &'static str| {
        let active = key == active_col;
        let next_dir = if active && active_dir == "desc" {
            "asc"
        } else {
            "desc"
        };
        SortHeaderView {
            label,
            class,
            hint,
            url: format!("{prefix}sort={key}&dir={next_dir}"),
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
    };
    RequestsSortHeaders {
        time: header(
            "created_at",
            "Time",
            "sp-col-date",
            "When the gateway accepted the call",
        ),
        tokens: header(
            "tokens",
            "Tokens",
            "sp-table__cell--num",
            "Input plus output tokens billed for this call",
        ),
        cost: header(
            "cost",
            "Cost",
            "sp-table__cell--num",
            "Billed cost, priced from the route's catalog entry",
        ),
        latency: header(
            "latency",
            "Latency",
            "sp-table__cell--num",
            "Wall-clock time from accept to last byte",
        ),
    }
}

pub(super) fn filter_options(query: &RequestsQuery, facets: &[FacetValue]) -> FilterOptionsView {
    let pick = |kind: &str, selected: Option<&str>, all_label: &str| {
        let mut out = vec![SelectOptionView {
            value: String::new(),
            label: all_label.to_owned(),
            selected: selected.is_none(),
        }];
        out.extend(
            facets
                .iter()
                .filter(|f| f.kind == kind)
                .map(|f| SelectOptionView {
                    selected: selected == Some(f.value.as_str()),
                    label: format!("{} ({})", f.value, f.count),
                    value: f.value.clone(),
                }),
        );
        out
    };
    let hidden = [
        ("tab", query.tab.as_deref()),
        ("preset", query.preset.as_deref()),
        ("from", query.from.as_deref()),
        ("to", query.to.as_deref()),
        ("group", query.group.as_deref()),
        ("project", query.project.as_deref()),
        ("sort", query.sort.as_deref()),
        ("dir", query.dir.as_deref()),
    ]
    .into_iter()
    .filter_map(|(name, value)| {
        value.filter(|v| !v.is_empty()).map(|v| HiddenFieldView {
            name: name.to_owned(),
            value: v.to_owned(),
        })
    })
    .collect();

    FilterOptionsView {
        models: pick(
            "model",
            empty_to_none(query.model.as_ref()).as_deref(),
            "All models",
        ),
        providers: pick(
            "provider",
            empty_to_none(query.provider.as_ref()).as_deref(),
            "All providers",
        ),
        statuses: pick(
            "status",
            empty_to_none(query.status.as_ref()).as_deref(),
            "All statuses",
        ),
        tools: pick(
            "tool",
            empty_to_none(query.tool.as_ref()).as_deref(),
            "All tools",
        ),
        hidden,
    }
}

pub(super) fn kpi_view(k: &RequestKpis, query: &RequestsQuery) -> RequestKpiView {
    let error_rate = k.failed as f64 / k.total.max(1) as f64 * 100.0;
    RequestKpiView {
        total: k.total,
        total_display: group_digits(k.total),
        unattributed: k.unattributed,
        attribution_sub: if k.unattributed > 0 {
            format!("{} unattributed", group_digits(k.unattributed))
        } else {
            "all attributed".to_owned()
        },
        cost_display: format_cost(k.cost_microdollars),
        tokens_sub: format!(
            "{} in / {} out",
            compact_int(k.input_tokens),
            compact_int(k.output_tokens)
        ),
        error_rate_display: format!("{error_rate:.1}%"),
        failed: k.failed,
        failed_sub: format!("{} failed calls", group_digits(k.failed)),
        has_failures: k.failed > 0,
        p95_display: format_duration_ms(k.p95_latency_ms.round() as i64),
        latency_sub: format!(
            "p50 {}",
            format_duration_ms(k.p50_latency_ms.round() as i64)
        ),
        rejected: k.rejected,
        rejected_url: log_filter_url(query, "status", "rejected"),
        rejected_active: query.status.as_deref() == Some("rejected"),
        denied: k.denied,
        tool_calls: k.tool_calls,
        tool_calls_sub: format!("{} pre-flight denies", group_digits(k.denied)),
    }
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

// Why: a four-digit request count read as a bare number is easy to misread by
// an order of magnitude in a dense grid of tiles.
pub(super) fn group_digits(v: i64) -> String {
    let digits = v.abs().to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    if v < 0 { format!("-{out}") } else { out }
}
