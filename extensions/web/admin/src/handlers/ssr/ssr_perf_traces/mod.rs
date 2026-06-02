//! `/admin/entities/traces` — Trace Explorer list page.
//!
//! Replaces the old plugin-events recap with a true trace list bound to the
//! shared time-range + identity-filter-ribbon URL contract. Each row links to
//! the per-trace waterfall at `/admin/entities/traces/{session_id}`.

use std::sync::Arc;

use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;

use crate::repositories::governance_grp::filter_options::fetch_filter_options;
use crate::repositories::governance_grp::time_range::{
    parse_time_range, TimeRange, TimeRangePreset, TimeRangeQuery,
};
use crate::repositories::perf_grp::traces::{
    fetch_trace_list, fetch_trace_stats, TraceFilter, TraceSort, TraceSortColumn, TraceSortDir,
    TraceStats,
};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

mod rows;
mod view;

const BASE_URL: &str = "/admin/entities/traces";
const PAGE_SIZE: i64 = 50;

#[derive(Debug, Deserialize)]
pub struct TraceListQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub preset: Option<String>,
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub agent_scope: Option<String>,
    pub policy: Option<String>,
    pub decision: Option<String>,
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
    let page = query.page.unwrap_or(0).max(0);

    let data = load_traces_data(&pool, &query, range, page).await;
    super::render_page(&engine, "perf-traces", &data, &user_ctx, &mkt_ctx)
}

async fn load_traces_data(
    pool: &PgPool,
    query: &TraceListQuery,
    range: TimeRange,
    page: i64,
) -> serde_json::Value {
    let preset = preset_str(query, range);
    let filter = TraceFilter {
        user_id: empty_to_none(query.user_id.as_deref()),
        agent_id: empty_to_none(query.agent_id.as_deref()),
        agent_scope: empty_to_none(query.agent_scope.as_deref()),
        policy: empty_to_none(query.policy.as_deref()),
        decision: empty_to_none(query.decision.as_deref()),
        error_only: query.error_only.as_deref() == Some("true"),
        deny_only: query.deny_only.as_deref() == Some("true"),
    };
    let sort = sort_from_query(query);
    let offset = page * PAGE_SIZE;
    let (list_res, stats_res, options_res) = tokio::join!(
        fetch_trace_list(pool, filter, range, sort, PAGE_SIZE, offset),
        fetch_trace_stats(pool, range),
        fetch_filter_options(pool, range),
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

    let total_pages = if total == 0 {
        1
    } else {
        (total + PAGE_SIZE - 1) / PAGE_SIZE
    };
    let pagination = view::build_pagination(query, page, total_pages);
    let view_qs = view::view_tabs_qs(range, &preset);

    json!({
        "page": "traces",
        "title": "Trace Explorer",
        "entity_view_tabs": view::entity_view_tabs("traces", &view_qs),
        "time_range": view::time_range_context(range, &preset),
        "filter_ribbon": {
            "base_url": BASE_URL,
            "preserved": view::build_preserved(query, range, &preset),
            "options": view::annotate_options(&options, &filter),
            "chips": view::build_chips(query),
        },
        "stats": view::serde_stats(&stats),
        "traces": rows.iter().map(rows::trace_to_json).collect::<Vec<_>>(),
        "has_traces": !rows.is_empty(),
        "total_count": total,
        "page_size": PAGE_SIZE,
        "page_index": page,
        "page_count": total_pages,
        "pagination": pagination,
        "sort": view::sort_col_to_str(sort.column),
        "dir": view::sort_dir_to_str(sort.dir),
        "error_only": filter.error_only,
        "deny_only": filter.deny_only,
    })
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
        Some("cost") => TraceSortColumn::Cost,
        Some("tokens") => TraceSortColumn::Tokens,
        _ => TraceSortColumn::StartedAt,
    };
    let dir = match query.dir.as_deref() {
        Some("asc") => TraceSortDir::Asc,
        _ => TraceSortDir::Desc,
    };
    TraceSort { column, dir }
}
