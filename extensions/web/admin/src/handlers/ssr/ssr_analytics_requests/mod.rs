//! `/admin/entities/requests` — Inference Requests gateway log.
//!
//! Reads the `/v1/messages` gateway spine from `ai_requests` (NOT
//! `plugin_usage_events`). Five URL-driven tabs over one window: Overview
//! (KPIs + traffic / cost / latency charts), Models / Providers / Status
//! (rollups that double as the filter picker), and Log (the paged call table).
//! Every log row carries `data-chain-id` pointing at the request id so the
//! chain-drawer can resolve it.

use crate::error::AdminError;
use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::AdminHtmlResult;
use crate::handlers::ssr::types as charts;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use crate::util::time_range::TimeRange;


mod context;
mod data;
mod urls;
mod view;

use context::{AnalyticsRequestsPageContext, RequestsTab};

const BASE_URL: &str = "/admin/entities/requests";
const PAGE_SIZE: i64 = 50;

#[derive(Debug, Deserialize)]
pub(crate) struct RequestsQuery {
    pub tab: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub preset: Option<String>,
    pub user_id: Option<systemprompt::identifiers::UserId>,
    pub agent_id: Option<systemprompt::identifiers::AgentId>,
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
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let tab = RequestsTab::from_query(query.tab.as_deref());
    let filter = view::filter_from_query(&query);
    let sort = view::sort_from_query(&query);
    let page = query.page.unwrap_or(0).max(0);
    let offset = page * PAGE_SIZE;

    let (range, auto_widened) = data::resolve_range(&pool, &query).await;

    let fetched = data::load_requests_data(
        &pool,
        data::RequestsPageQuery {
            tab,
            filter: &filter,
            range,
            sort,
            page_size: PAGE_SIZE,
            offset,
        },
    )
    .await;

    let ctx = page_context(PageInput {
        query: &query,
        tab,
        filter: &filter,
        range,
        auto_widened,
        page,
        fetched: &fetched,
    });

    Ok(super::render_typed_page(
        &engine,
        "analytics-requests",
        &ctx,
        &user_ctx,
        &mkt_ctx,
    ))
}

// Why: the handler owns auth and I/O; assembling the template context is a pure
// function of what came back, and keeps either half readable on its own.
#[derive(Clone, Copy)]
struct PageInput<'a> {
    query: &'a RequestsQuery,
    tab: RequestsTab,
    filter: &'a crate::repositories::analytics::requests::RequestFilter,
    range: TimeRange,
    auto_widened: Option<&'static str>,
    page: i64,
    fetched: &'a data::RequestsData,
}

fn page_context(input: PageInput<'_>) -> AnalyticsRequestsPageContext {
    let PageInput {
        query,
        tab,
        filter,
        range,
        auto_widened,
        page,
        fetched,
    } = input;
    let pagination = urls::build_pagination(
        query,
        crate::handlers::ssr::list_view::PageWindow::new(
            page,
            PAGE_SIZE,
            fetched.total_count,
            i64::try_from(fetched.rows.len()).unwrap_or(PAGE_SIZE),
            "requests",
        ),
    );
    let search_query = query.q.clone().unwrap_or_default();
    let has_active_filters = filter.model.is_some()
        || filter.provider.is_some()
        || filter.status.is_some()
        || !search_query.is_empty();

    AnalyticsRequestsPageContext {
        page: "requests",
        title: "Inference Requests",
        time_range: view::time_range_context(query, &range, auto_widened),
        tabs: urls::tab_links(tab, query, fetched.total_count),
        is_overview: tab == RequestsTab::Overview,
        is_breakdown: matches!(
            tab,
            RequestsTab::Models | RequestsTab::Providers | RequestsTab::Status
        ),
        is_log: tab == RequestsTab::Log,
        stats: view::stats_to_json(&fetched.stats),
        histogram: charts::histogram_view(&fetched.hist, &fetched.stats),
        traffic_chart: charts::traffic_chart(&fetched.series, &range),
        cost_chart: charts::cost_chart(&fetched.series, &range),
        breakdown: view::breakdown_view(tab, &fetched.breakdown, query),
        rows: fetched.rows.iter().map(view::request_row_to_json).collect(),
        has_rows: !fetched.rows.is_empty(),
        total_count: fetched.total_count,
        pagination,
        search_query,
        chips: urls::active_chips(query),
        has_active_filters,
        clear_url: urls::clear_url(query),
        base_url: BASE_URL,
    }
}
