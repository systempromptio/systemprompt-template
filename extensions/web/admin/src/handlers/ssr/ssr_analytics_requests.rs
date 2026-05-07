//! `/admin/entities/requests` — Inference Requests gateway log.
//!
//! Reads the `/v1/messages` gateway spine from `ai_requests` (NOT
//! `plugin_usage_events`). KPI strip + latency histogram + cost-over-time +
//! filterable / sortable paged table. Every row carries `data-chain-id`
//! pointing at the request id so the chain-drawer can resolve it.

use std::sync::Arc;

use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use urlencoding::encode as urlencode;

use crate::repositories::analytics_grp::request_stats::{
    fetch_cost_over_time, fetch_latency_histogram, fetch_request_stats, CostBucket, LatencyBucket,
    RequestStats,
};
use crate::repositories::analytics_grp::requests::{
    fetch_request_filter_options, fetch_requests_paged, RequestFilter, RequestFilterOptions,
    RequestRow, RequestSortColumn, RequestSortSpec, SortDir,
};
use crate::repositories::governance_grp::time_range::{
    count_requests_in_range, parse_time_range, preset_to_range, TimeRangePreset, TimeRangeQuery,
};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

const BASE_URL: &str = "/admin/entities/requests";
const PAGE_SIZE: i64 = 50;

#[derive(Debug, Deserialize)]
pub struct RequestsQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub preset: Option<String>,
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub status: Option<String>,
    pub q: Option<String>,
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub page: Option<i64>,
}

pub async fn analytics_requests_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<RequestsQuery>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let user_picked_range = query.preset.is_some()
        || (query.from.is_some() && query.to.is_some());
    let initial_range = parse_time_range(&TimeRangeQuery {
        from: query.from.clone(),
        to: query.to.clone(),
        preset: query.preset.clone(),
    });
    let filter = filter_from_query(&query);
    let sort = sort_from_query(&query);
    let page = query.page.unwrap_or(0).max(0);
    let offset = page * PAGE_SIZE;

    // Sensible default: when the user did not pick a window and the default
    // 24h is empty, widen progressively (7d -> 30d) so the page actually shows
    // data. If the user explicitly chose a preset, respect it.
    let (range, auto_widened): (_, Option<&'static str>) = if user_picked_range {
        (initial_range, None)
    } else {
        let mut chosen = initial_range;
        let mut widened: Option<&'static str> = None;
        for (label, preset) in [
            ("24h", TimeRangePreset::Hours24),
            ("7d", TimeRangePreset::Days7),
            ("30d", TimeRangePreset::Days30),
        ] {
            let candidate = preset_to_range(preset);
            let count = count_requests_in_range(&pool, candidate).await.unwrap_or(0);
            if count > 0 {
                chosen = candidate;
                widened = if label == "24h" { None } else { Some(label) };
                break;
            }
        }
        (chosen, widened)
    };

    let (paged, stats_res, hist_res, cost_res, options_res) = tokio::join!(
        fetch_requests_paged(&pool, &filter, range, sort, PAGE_SIZE, offset),
        fetch_request_stats(&pool, range),
        fetch_latency_histogram(&pool, range),
        fetch_cost_over_time(&pool, range),
        fetch_request_filter_options(&pool, range),
    );

    let (rows, total_count) = paged.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_requests_paged failed");
        (Vec::new(), 0)
    });
    let stats = stats_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_request_stats failed");
        RequestStats::default()
    });
    let hist = hist_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_latency_histogram failed");
        Vec::new()
    });
    let cost = cost_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_cost_over_time failed");
        Vec::new()
    });
    let options = options_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_request_filter_options failed");
        RequestFilterOptions::default()
    });

    let total_pages = if total_count == 0 {
        1
    } else {
        (total_count + PAGE_SIZE - 1) / PAGE_SIZE
    };
    let pagination = build_pagination(&query, page, total_pages);
    let search_query = query.q.clone().unwrap_or_default();
    let has_active_filters = filter.model.is_some()
        || filter.provider.is_some()
        || filter.status.is_some()
        || !search_query.is_empty();

    let data = json!({
        "page": "requests",
        "title": "Inference Requests",
        "time_range": time_range_context(&query, &range, auto_widened),
        "stats": stats_to_json(&stats),
        "histogram": hist.iter().map(latency_bucket_to_json).collect::<Vec<_>>(),
        "histogram_max": hist.iter().map(|b| b.count).max().unwrap_or(0),
        "cost_series": cost.iter().map(cost_bucket_to_json).collect::<Vec<_>>(),
        "cost_max": cost.iter().map(|b| b.cost_microdollars).max().unwrap_or(0),
        "rows": rows.iter().map(request_row_to_json).collect::<Vec<_>>(),
        "has_rows": !rows.is_empty(),
        "total_count": total_count,
        "pagination": pagination,
        "search_query": search_query,
        "filters": filters_to_json(&filter, &options),
        "has_active_filters": has_active_filters,
        "clear_url": clear_url(&query),
        "base_url": BASE_URL,
    });

    super::render_page(&engine, "analytics-requests", &data, &user_ctx, &mkt_ctx)
}

fn filter_from_query(query: &RequestsQuery) -> RequestFilter {
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
        .map(ToString::to_string)
}

fn sort_from_query(query: &RequestsQuery) -> RequestSortSpec {
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

fn stats_to_json(s: &RequestStats) -> serde_json::Value {
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

fn latency_bucket_to_json(b: &LatencyBucket) -> serde_json::Value {
    json!({
        "label": b.label,
        "count": b.count,
        "upper_bound_ms": b.upper_bound_ms,
    })
}

fn cost_bucket_to_json(b: &CostBucket) -> serde_json::Value {
    json!({
        "bucket_index": b.bucket_index,
        "bucket_start": b.bucket_start.to_rfc3339(),
        "cost_microdollars": b.cost_microdollars,
    })
}

fn request_row_to_json(r: &RequestRow) -> serde_json::Value {
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
        return "—".to_string();
    };
    let dollars = m as f64 / 1_000_000.0;
    if dollars == 0.0 {
        "$0".to_string()
    } else if dollars < 0.01 {
        format!("${dollars:.6}")
    } else {
        format!("${dollars:.4}")
    }
}

fn time_range_context(
    query: &RequestsQuery,
    range: &crate::repositories::governance_grp::time_range::TimeRange,
    auto_widened: Option<&'static str>,
) -> serde_json::Value {
    let preset = query.preset.clone().unwrap_or_else(|| {
        if query.from.is_some() && query.to.is_some() {
            "custom".to_string()
        } else {
            auto_widened.unwrap_or("24h").to_string()
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

fn filters_to_json(filter: &RequestFilter, options: &RequestFilterOptions) -> serde_json::Value {
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

fn clear_url(query: &RequestsQuery) -> String {
    // Preserve only the time-range params. Drop filters, search, sort, page.
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
        BASE_URL.to_string()
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
    if !drop.contains(&"page") {
        if let Some(p) = query.page.filter(|p| *p > 0) {
            parts.push(format!("page={p}"));
        }
    }
    parts.join("&")
}

fn build_pagination(query: &RequestsQuery, page: i64, total_pages: i64) -> serde_json::Value {
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

