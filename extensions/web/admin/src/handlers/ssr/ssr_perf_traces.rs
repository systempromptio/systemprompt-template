//! `/admin/performance/traces` — Trace Explorer list page.
//!
//! Replaces the old plugin-events recap with a true trace list bound to the
//! shared time-range + identity-filter-ribbon URL contract. Each row links to
//! the per-trace waterfall at `/admin/performance/traces/{session_id}`.

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

use crate::repositories::governance_grp::filter_options::{
    fetch_filter_options, FilterOption, FilterOptions,
};
use crate::repositories::governance_grp::time_range::{
    parse_time_range, TimeRange, TimeRangePreset, TimeRangeQuery,
};
use crate::repositories::perf_grp::traces::{
    fetch_trace_list, fetch_trace_stats, TraceFilter, TraceSort, TraceSortColumn, TraceSortDir,
    TraceStats, TraceSummary,
};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

const BASE_URL: &str = "/admin/performance/traces";
const PAGE_SIZE: i64 = 50;

#[derive(Debug, Deserialize)]
pub struct TraceListQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub preset: Option<String>,
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub agent_scope: Option<String>,
    pub error_only: Option<String>,
    pub deny_only: Option<String>,
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub page: Option<i64>,
}

pub async fn perf_traces_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<TraceListQuery>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let range = parse_time_range(&TimeRangeQuery {
        from: query.from.clone(),
        to: query.to.clone(),
        preset: query.preset.clone(),
    });
    let preset = preset_str(&query, range);
    let filter = TraceFilter {
        user_id: empty_to_none(query.user_id.as_deref()),
        agent_id: empty_to_none(query.agent_id.as_deref()),
        agent_scope: empty_to_none(query.agent_scope.as_deref()),
        error_only: query.error_only.as_deref() == Some("true"),
        deny_only: query.deny_only.as_deref() == Some("true"),
    };
    let sort = sort_from_query(&query);
    let page = query.page.unwrap_or(0).max(0);
    let offset = page * PAGE_SIZE;

    let (list_res, stats_res, options_res) = tokio::join!(
        fetch_trace_list(&pool, filter, range, sort, PAGE_SIZE, offset),
        fetch_trace_stats(&pool, range),
        fetch_filter_options(&pool, range),
    );

    let (rows, total) = list_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_trace_list failed");
        (Vec::new(), 0)
    });
    let stats = stats_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_trace_stats failed");
        TraceStats::default()
    });
    let options = options_res.unwrap_or_default();

    let total_pages = if total == 0 { 1 } else { (total + PAGE_SIZE - 1) / PAGE_SIZE };
    let pagination = build_pagination(&query, page, total_pages);

    let data = json!({
        "page": "perf-traces",
        "title": "Trace Explorer",
        "time_range": time_range_context(range, &preset),
        "filter_ribbon": {
            "base_url": BASE_URL,
            "preserved": build_preserved(&query, range, &preset),
            "options": annotate_options(&options, &filter),
            "chips": build_chips(&query),
        },
        "stats": serde_stats(&stats),
        "traces": rows.iter().map(trace_to_json).collect::<Vec<_>>(),
        "has_traces": !rows.is_empty(),
        "total_count": total,
        "page_size": PAGE_SIZE,
        "page_index": page,
        "page_count": total_pages,
        "pagination": pagination,
        "sort": sort_col_to_str(sort.column),
        "dir": sort_dir_to_str(sort.dir),
        "error_only": filter.error_only,
        "deny_only": filter.deny_only,
    });

    super::render_page(&engine, "perf-traces", &data, &user_ctx, &mkt_ctx)
}

fn empty_to_none(v: Option<&str>) -> Option<&str> {
    v.filter(|s| !s.is_empty())
}

fn preset_str(query: &TraceListQuery, range: TimeRange) -> String {
    if let Some(p) = query.preset.as_deref() {
        if !p.is_empty() {
            return p.to_string();
        }
    }
    if query.from.is_some() && query.to.is_some() {
        return "custom".to_string();
    }
    match range.preset {
        TimeRangePreset::Min15 => "15m",
        TimeRangePreset::Hour1 => "1h",
        TimeRangePreset::Hours24 => "24h",
        TimeRangePreset::Days7 => "7d",
        TimeRangePreset::Days30 => "30d",
        TimeRangePreset::Custom => "custom",
    }
    .to_string()
}

fn sort_from_query(query: &TraceListQuery) -> TraceSort {
    let column = match query.sort.as_deref() {
        Some("duration") => TraceSortColumn::Duration,
        Some("spans") => TraceSortColumn::SpanCount,
        _ => TraceSortColumn::StartedAt,
    };
    let dir = match query.dir.as_deref() {
        Some("asc") => TraceSortDir::Asc,
        _ => TraceSortDir::Desc,
    };
    TraceSort { column, dir }
}

const fn sort_col_to_str(c: TraceSortColumn) -> &'static str {
    match c {
        TraceSortColumn::StartedAt => "started_at",
        TraceSortColumn::Duration => "duration",
        TraceSortColumn::SpanCount => "spans",
    }
}

const fn sort_dir_to_str(d: TraceSortDir) -> &'static str {
    match d {
        TraceSortDir::Asc => "asc",
        TraceSortDir::Desc => "desc",
    }
}

fn time_range_context(range: TimeRange, preset: &str) -> serde_json::Value {
    json!({
        "preset": preset,
        "from": range.from.to_rfc3339(),
        "to": range.to.to_rfc3339(),
        "base_url": BASE_URL,
        "query": "",
    })
}

fn build_preserved(
    query: &TraceListQuery,
    range: TimeRange,
    preset: &str,
) -> Vec<serde_json::Value> {
    let mut out = vec![
        json!({ "name": "preset", "value": preset }),
        json!({ "name": "from",   "value": range.from.to_rfc3339() }),
        json!({ "name": "to",     "value": range.to.to_rfc3339() }),
    ];
    if query.error_only.as_deref() == Some("true") {
        out.push(json!({ "name": "error_only", "value": "true" }));
    }
    if query.deny_only.as_deref() == Some("true") {
        out.push(json!({ "name": "deny_only", "value": "true" }));
    }
    out
}

fn build_chips(query: &TraceListQuery) -> Vec<serde_json::Value> {
    const GROUPS: &[(&str, &str)] = &[
        ("user_id", "User"),
        ("agent_id", "Agent"),
        ("agent_scope", "Scope"),
    ];
    let mut chips = Vec::new();
    for (param, label) in GROUPS {
        let val = match *param {
            "user_id" => query.user_id.as_deref(),
            "agent_id" => query.agent_id.as_deref(),
            "agent_scope" => query.agent_scope.as_deref(),
            _ => None,
        };
        let Some(v) = empty_to_none(val) else { continue };
        chips.push(json!({
            "group_label": label,
            "label": v,
            "value": v,
            "remove_url": chip_remove_url(query, param),
        }));
    }
    chips
}

fn chip_remove_url(query: &TraceListQuery, drop: &str) -> String {
    let qs = preserved_query_string(query, &[drop]);
    if qs.is_empty() {
        BASE_URL.to_string()
    } else {
        format!("{BASE_URL}?{qs}")
    }
}

fn preserved_query_string(query: &TraceListQuery, drop: &[&str]) -> String {
    let pairs: [(&str, Option<&str>); 10] = [
        ("preset", query.preset.as_deref()),
        ("from", query.from.as_deref()),
        ("to", query.to.as_deref()),
        ("user_id", query.user_id.as_deref()),
        ("agent_id", query.agent_id.as_deref()),
        ("agent_scope", query.agent_scope.as_deref()),
        ("error_only", query.error_only.as_deref()),
        ("deny_only", query.deny_only.as_deref()),
        ("sort", query.sort.as_deref()),
        ("dir", query.dir.as_deref()),
    ];
    pairs
        .iter()
        .filter(|(name, _)| !drop.contains(name))
        .filter_map(|(name, val)| {
            val.filter(|s| !s.is_empty())
                .map(|v| format!("{}={}", name, urlencode(v)))
        })
        .collect::<Vec<_>>()
        .join("&")
}

fn annotate_options(
    options: &FilterOptions,
    filter: &TraceFilter<'_>,
) -> serde_json::Value {
    json!({
        "users":        annotate_group(&options.users,        filter.user_id),
        "agents":       annotate_group(&options.agents,       filter.agent_id),
        "agent_scopes": annotate_group(&options.agent_scopes, filter.agent_scope),
        "policies":     Vec::<serde_json::Value>::new(),
        "decisions":    Vec::<serde_json::Value>::new(),
    })
}

fn annotate_group(items: &[FilterOption], selected: Option<&str>) -> Vec<serde_json::Value> {
    items
        .iter()
        .map(|o| {
            json!({
                "id": o.id,
                "label": o.label,
                "count": o.count,
                "selected": selected.is_some_and(|s| s == o.id),
            })
        })
        .collect()
}

fn build_pagination(
    query: &TraceListQuery,
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

fn serde_stats(s: &TraceStats) -> serde_json::Value {
    json!({
        "total_traces": s.total_traces,
        "error_count": s.error_count,
        "deny_count": s.deny_count,
        "p50_ms": s.p50_duration_ms,
        "p95_ms": s.p95_duration_ms,
        "p99_ms": s.p99_duration_ms,
    })
}

fn trace_to_json(t: &TraceSummary) -> serde_json::Value {
    let short = if t.session_id.len() > 16 {
        t.session_id[..16].to_string()
    } else {
        t.session_id.clone()
    };
    json!({
        "session_id": t.session_id,
        "session_id_short": short,
        "trace_id": t.trace_id,
        "started_at": t.started_at.to_rfc3339(),
        "started_at_local": t
            .started_at
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
        "duration_ms": t.duration_ms,
        "duration_display": format_duration(t.duration_ms),
        "user_id": t.user_id,
        "agent_id": t.agent_id,
        "model": t.model,
        "span_count": t.span_count,
        "has_error": t.has_error,
        "has_deny": t.has_deny,
        "detail_url": format!("{BASE_URL}/{}", urlencode(&t.session_id)),
    })
}

fn format_duration(ms: i64) -> String {
    if ms < 1000 {
        format!("{ms} ms")
    } else if ms < 60_000 {
        format!("{:.2} s", ms as f64 / 1000.0)
    } else {
        format!("{:.1} min", ms as f64 / 60_000.0)
    }
}
