//! `/admin/entities/requests` — Inference Requests gateway log.
//!
//! Reads the `/v1/messages` gateway spine from `ai_requests` (NOT
//! `plugin_usage_events`). KPI strip + latency histogram + cost-over-time +
//! filterable / sortable paged table. Every row carries `data-chain-id`
//! pointing at the request id so the chain-drawer can resolve it.

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use serde::Deserialize;
use sqlx::PgPool;

use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

mod context;
mod data;
mod view;

use context::AnalyticsRequestsPageContext;

const BASE_URL: &str = "/admin/entities/requests";
const PAGE_SIZE: i64 = 50;

#[derive(Debug, Deserialize)]
pub(crate) struct RequestsQuery {
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

pub(crate) async fn analytics_requests_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<RequestsQuery>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let filter = view::filter_from_query(&query);
    let sort = view::sort_from_query(&query);
    let page = query.page.unwrap_or(0).max(0);
    let offset = page * PAGE_SIZE;

    let (range, auto_widened) = data::resolve_range(&pool, &query).await;

    let fetched = data::fetch_requests_data(
        &pool,
        data::RequestsPageQuery {
            filter: &filter,
            range,
            sort,
            page_size: PAGE_SIZE,
            offset,
        },
    )
    .await;

    let total_pages = if fetched.total_count == 0 {
        1
    } else {
        (fetched.total_count + PAGE_SIZE - 1) / PAGE_SIZE
    };
    let pagination = view::build_pagination(&query, page, total_pages);
    let search_query = query.q.clone().unwrap_or_default();
    let has_active_filters = filter.model.is_some()
        || filter.provider.is_some()
        || filter.status.is_some()
        || !search_query.is_empty();

    let ctx = AnalyticsRequestsPageContext {
        page: "requests",
        title: "Inference Requests",
        time_range: view::time_range_context(&query, &range, auto_widened),
        stats: view::stats_to_json(&fetched.stats),
        histogram: fetched
            .hist
            .iter()
            .map(view::latency_bucket_to_json)
            .collect(),
        histogram_max: fetched.hist.iter().map(|b| b.count).max().unwrap_or(0),
        cost_series: fetched.cost.iter().map(view::cost_bucket_to_json).collect(),
        cost_max: fetched
            .cost
            .iter()
            .map(|b| b.cost_microdollars)
            .max()
            .unwrap_or(0),
        rows: fetched.rows.iter().map(view::request_row_to_json).collect(),
        has_rows: !fetched.rows.is_empty(),
        total_count: fetched.total_count,
        pagination,
        search_query,
        filters: view::filters_to_json(&filter, &fetched.options),
        has_active_filters,
        clear_url: view::clear_url(&query),
        base_url: BASE_URL,
    };

    super::render_typed_page(&engine, "analytics-requests", &ctx, &user_ctx, &mkt_ctx)
}
