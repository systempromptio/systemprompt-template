//! `/admin/requests` — the inference request log.
//!
//! Reads the `/v1/messages` gateway spine from `ai_requests` (NOT
//! `plugin_usage_events`). The Log tab is the page: every call in the window,
//! attributed exclusively to one project and one group, with its governance
//! decision and tool-call count on the row. Four sibling tabs roll the same
//! window up by model, provider and status, or draw it as traffic, cost and
//! latency; each doubles as the fastest way to pick a filter.

use crate::error::AdminError;
use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::{AdminHtmlResult, AdminResult};
use crate::handlers::ssr::csv::{CsvBuilder, usd};
use crate::handlers::ssr::types as charts;
use crate::handlers::ssr::types::BreadcrumbView;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use crate::util::time_range::TimeRange;

mod context;
mod data;
mod summary;
mod urls;
mod view;

use context::{AnalyticsRequestsPageContext, RequestsTab};

const BASE_URL: &str = "/admin/requests";
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
    pub tool: Option<String>,
    pub q: Option<String>,
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub page: Option<i64>,
    pub group: Option<String>,
    pub project: Option<String>,
    // Why: the shorter spellings the analytics drill-downs emit. They are read
    // once, folded into the canonical parameter, and never serialised again —
    // so every link this page builds speaks one vocabulary and a shared URL
    // does not depend on which page produced it.
    pub user: Option<systemprompt::identifiers::UserId>,
    pub range: Option<String>,
    pub scope: Option<String>,
}

impl RequestsQuery {
    fn normalize(&mut self) {
        if self.user_id.is_none() {
            self.user_id = self.user.take();
        }
        if self.preset.is_none() {
            self.preset = self.range.take().filter(|s| !s.is_empty());
        }
        // Why: `scope=group:commerce-devs` / `scope=project:checkout`. A bare
        // or unprefixed value names nothing this page can resolve, so it is
        // ignored rather than guessed at.
        if let Some(raw) = self.scope.take() {
            match raw.split_once(':') {
                Some(("group", id)) if self.group.is_none() => {
                    self.group = Some(id.to_owned());
                },
                Some(("project", id)) if self.project.is_none() => {
                    self.project = Some(id.to_owned());
                },
                _ => {},
            }
        }
    }
}

pub(crate) async fn analytics_requests_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(mut query): Query<RequestsQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    query.normalize();

    let tab = RequestsTab::from_query(query.tab.as_deref());
    let request = crate::repositories::scope::ScopeRequest::from_query(
        &user_ctx,
        query.group.as_deref(),
        query.project.as_deref(),
    );
    let scope = crate::repositories::scope::membership::get_subject_scope(&pool, &request).await?;
    let filter = view::filter_from_query(&query, scope);
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

    let scope_filter = view::scope_filter(&pool, &user_ctx, &request, &query).await;
    let ctx = page_context(PageInput {
        scope_filter,
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

// Why: `/admin/requests.csv` — the rows the log is showing, as a download.
pub(crate) async fn analytics_requests_csv(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Query(mut query): Query<RequestsQuery>,
) -> AdminResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()));
    }
    query.normalize();

    let request = crate::repositories::scope::ScopeRequest::from_query(
        &user_ctx,
        query.group.as_deref(),
        query.project.as_deref(),
    );
    let scope = crate::repositories::scope::membership::get_subject_scope(&pool, &request).await?;
    let filter = view::filter_from_query(&query, scope);
    let (range, _) = data::resolve_range(&pool, &query).await;

    // Why: one page's worth is not an export. The cap is high enough to be a
    // real answer and low enough that a mis-set window cannot stream the whole
    // audit table into a browser.
    let page = crate::repositories::analytics::requests::RequestPage {
        sort: view::sort_from_query(&query),
        limit: 5000,
        offset: 0,
    };
    let (rows, _) =
        crate::repositories::analytics::requests::list_requests_paged(&pool, &filter, range, page)
            .await
            .unwrap_or_default();

    let mut csv = CsvBuilder::new(&[
        "created_at",
        "request_id",
        "user_id",
        "group_id",
        "project_id",
        "provider",
        "model",
        "status",
        "input_tokens",
        "output_tokens",
        "cost_usd",
        "latency_ms",
        "tool_calls",
        "deny_count",
    ]);
    for r in &rows {
        csv.row(&[
            &r.created_at.to_rfc3339(),
            r.request_id.as_str(),
            r.user_id.as_str(),
            r.group_id.as_deref().unwrap_or("unattributed"),
            r.project_id.as_deref().unwrap_or("unattributed"),
            &r.provider,
            &r.model,
            &r.status,
            &r.input_tokens.unwrap_or(0).to_string(),
            &r.output_tokens.unwrap_or(0).to_string(),
            &usd(r.cost_microdollars),
            &r.latency_ms.unwrap_or(0).to_string(),
            &r.tool_call_count.to_string(),
            &r.deny_count.to_string(),
        ]);
    }
    Ok(csv.into_response("requests.csv"))
}

// Why: the handler owns auth and I/O; assembling the template context is a pure
// function of what came back, and keeps either half readable on its own.
struct PageInput<'a> {
    scope_filter: crate::handlers::ssr::list_view::ScopeFilterView,
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
        scope_filter,
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
        || filter.tool.is_some()
        || !search_query.is_empty();

    AnalyticsRequestsPageContext {
        page: "requests",
        title: "Inference Requests",
        breadcrumbs: vec![
            BreadcrumbView::link("AI activity", "/admin/analytics"),
            BreadcrumbView::current("Requests"),
        ],
        time_range: view::time_range_context(query, &range, auto_widened),
        tabs: urls::tab_links(tab, query, fetched.total_count),
        is_overview: tab == RequestsTab::Overview,
        is_breakdown: matches!(
            tab,
            RequestsTab::Models | RequestsTab::Providers | RequestsTab::Status
        ),
        is_log: tab == RequestsTab::Log,
        kpis: summary::kpi_view(&fetched.kpis, query),
        stats: summary::stats_to_json(&fetched.stats),
        histogram: charts::histogram_view(&fetched.hist, &fetched.stats),
        traffic_chart: charts::traffic_chart(&fetched.series, &range),
        cost_chart: charts::cost_chart(&fetched.series, &range),
        breakdown: summary::breakdown_view(tab, &fetched.breakdown, query),
        filters: summary::filter_options(query, &fetched.facets),
        sort_headers: summary::sort_headers(query),
        rows: fetched.rows.iter().map(view::request_row_to_json).collect(),
        has_rows: !fetched.rows.is_empty(),
        rows_unavailable: fetched.rows_unavailable,
        total_count: fetched.total_count,
        row_count_label: view::row_count_label(fetched.kpis.total),
        pagination,
        search_query,
        chips: urls::active_chips(query),
        has_active_filters,
        clear_url: urls::clear_url(query),
        csv_url: urls::csv_url(query),
        base_url: BASE_URL,
        scope_filter,
    }
}
