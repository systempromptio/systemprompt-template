//! View-model assembly for the Inference Requests page.
//!
//! Pure functions that turn repository rows + the parsed query into the serde
//! JSON the `analytics-requests` template consumes: KPI strip, histogram /
//! cost series, paged rows, filter options, and the URL builders that preserve
//! query state across pagination and the time-range presets.

use serde_json::json;
use urlencoding::encode as urlencode;

use crate::repositories::analytics_grp::request_stats::{CostBucket, LatencyBucket, RequestStats};
use crate::repositories::analytics_grp::requests::{
    RequestFilter, RequestFilterOptions, RequestRow, RequestSortColumn, RequestSortSpec, SortDir,
};
use crate::repositories::governance_grp::time_range::TimeRange;

use super::{BASE_URL, RequestsQuery};

pub(super) fn filter_from_query(query: &RequestsQuery) -> RequestFilter {
    RequestFilter {
        user_id: empty_to_none(query.user_id.as_ref()),
        agent_id: empty_to_none(query.agent_id.as_ref()),
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

pub(super) fn stats_to_json(s: &RequestStats) -> serde_json::Value {
    json!({
        "total": s.total,
        "error_count": s.error_count,
        "requests_per_minute": format!("{:.2}", s.requests_per_minute),
        "p50_latency_ms": s.p50_latency_ms.round() as i64,
        "p95_latency_ms": s.p95_latency_ms.round() as i64,
        "p99_latency_ms": s.p99_latency_ms.round() as i64,
        "total_cost_display": format_cost(Some(s.total_cost_microdollars)),
        "error_rate_pct": format!("{:.2}", s.error_rate * 100.0),
        "denied_session_count": s.denied_session_count,
        "denied_session_rate_pct": format!("{:.2}", s.denied_session_rate * 100.0),
    })
}

pub(super) fn latency_bucket_to_json(b: &LatencyBucket) -> serde_json::Value {
    json!({
        "label": b.label,
        "count": b.count,
        "upper_bound_ms": b.upper_bound_ms,
    })
}

pub(super) fn cost_bucket_to_json(b: &CostBucket) -> serde_json::Value {
    json!({
        "bucket_index": b.bucket_index,
        "bucket_start": b.bucket_start.to_rfc3339(),
        "cost_microdollars": b.cost_microdollars,
    })
}

pub(super) fn request_row_to_json(r: &RequestRow) -> serde_json::Value {
    json!({
        "id": r.id,
        "request_id": r.request_id,
        "trace_id": r.trace_id,
        "session_id": r.session_id,
        "user_id": r.user_id,
        "user_label": r.user_label.clone().unwrap_or_else(|| r.user_id.clone()),
        "provider": r.provider,
        "model": r.model,
        "status": r.status,
        "is_error": is_error_status(&r.status),
        "input_tokens": r.input_tokens,
        "output_tokens": r.output_tokens,
        "tokens_total": r.input_tokens.unwrap_or(0) + r.output_tokens.unwrap_or(0),
        "cost_microdollars": r.cost_microdollars,
        "cost_display": format_cost(Some(r.cost_microdollars)),
        "latency_ms": r.latency_ms,
        "error_message": r.error_message,
        "decision_count": r.decision_count,
        "deny_count": r.deny_count,
        "is_denied_preflight": r.deny_count > 0,
        "tool_call_count": r.tool_call_count,
        "created_at": r.created_at.to_rfc3339(),
        "created_at_local": r
            .created_at
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
    })
}

fn is_error_status(status: &str) -> bool {
    !matches!(status, "completed" | "pending" | "streaming")
}

fn format_cost(microdollars: Option<i64>) -> String {
    let Some(m) = microdollars else {
        return "—".to_owned();
    };
    let dollars = m as f64 / 1_000_000.0;
    if dollars == 0.0 {
        "$0".to_owned()
    } else if dollars < 0.01 {
        format!("${dollars:.6}")
    } else {
        format!("${dollars:.4}")
    }
}

pub(super) fn time_range_context(
    query: &RequestsQuery,
    range: &TimeRange,
    auto_widened: Option<&'static str>,
) -> serde_json::Value {
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
    json!({
        "preset": preset,
        "from": range.from.to_rfc3339(),
        "to": range.to.to_rfc3339(),
        "base_url": BASE_URL,
        "query": q_suffix,
        "auto_widened": auto_widened,
    })
}

pub(super) fn filters_to_json(
    filter: &RequestFilter,
    options: &RequestFilterOptions,
) -> serde_json::Value {
    json!({
        "model": filter.model,
        "provider": filter.provider,
        "status": filter.status,
        "options": {
            "models": options.models,
            "providers": options.providers,
            "statuses": options.statuses,
        },
    })
}

pub(super) fn clear_url(query: &RequestsQuery) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(p) = query.preset.as_deref().filter(|s| !s.is_empty()) {
        parts.push(format!("preset={}", urlencode(p)));
    }
    if let Some(f) = query.from.as_deref().filter(|s| !s.is_empty()) {
        parts.push(format!("from={}", urlencode(f)));
    }
    if let Some(t) = query.to.as_deref().filter(|s| !s.is_empty()) {
        parts.push(format!("to={}", urlencode(t)));
    }
    if parts.is_empty() {
        BASE_URL.to_owned()
    } else {
        format!("{BASE_URL}?{}", parts.join("&"))
    }
}

fn preserved_query_string(query: &RequestsQuery, drop: &[&str]) -> String {
    let mut parts: Vec<String> = Vec::new();
    let pairs_str: [(&str, Option<&str>); 11] = [
        ("preset", query.preset.as_deref()),
        ("from", query.from.as_deref()),
        ("to", query.to.as_deref()),
        ("user_id", query.user_id.as_deref()),
        ("agent_id", query.agent_id.as_deref()),
        ("model", query.model.as_deref()),
        ("provider", query.provider.as_deref()),
        ("status", query.status.as_deref()),
        ("q", query.q.as_deref()),
        ("sort", query.sort.as_deref()),
        ("dir", query.dir.as_deref()),
    ];
    for (name, value) in pairs_str {
        if drop.contains(&name) {
            continue;
        }
        let Some(v) = value.filter(|s| !s.is_empty()) else {
            continue;
        };
        parts.push(format!("{}={}", name, urlencode(v)));
    }
    if !drop.contains(&"page")
        && let Some(p) = query.page.filter(|p| *p > 0)
    {
        parts.push(format!("page={p}"));
    }
    parts.join("&")
}

pub(super) fn build_pagination(
    query: &RequestsQuery,
    page: i64,
    total_pages: i64,
) -> serde_json::Value {
    let qs = preserved_query_string(query, &["page"]);
    let prefix = if qs.is_empty() {
        format!("{BASE_URL}?")
    } else {
        format!("{BASE_URL}?{qs}&")
    };
    let prev_url = (page > 0).then(|| format!("{prefix}page={}", page - 1));
    let next_url = (page + 1 < total_pages).then(|| format!("{prefix}page={}", page + 1));
    json!({
        "current_page": page + 1,
        "total_pages": total_pages,
        "has_prev": prev_url.is_some(),
        "has_next": next_url.is_some(),
        "prev_url": prev_url,
        "next_url": next_url,
    })
}
