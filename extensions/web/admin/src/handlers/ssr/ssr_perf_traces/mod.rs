//! `/admin/traces` — Trace Explorer list page.
//!
//! Replaces the old plugin-events recap with a true trace list bound to the
//! shared time-range + identity-filter-ribbon URL contract. Each row links to
//! the per-trace waterfall at `/admin/traces/{session_id}`.

use crate::error::AdminError;
use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::{AgentId, UserId};

use crate::error::AdminHtmlResult;
use crate::handlers::ssr::list_view;
use crate::repositories::governance::filter_options::get_filter_options;
use crate::repositories::scope::{ScopeRequest, SubjectScope};
use crate::repositories::traces::{
    TraceFilter, TracePage, TraceSort, TraceSortColumn, TraceSortDir, TraceStats, get_trace_stats,
    list_traces,
};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use crate::util::time_range::{TimeRange, TimeRangePreset, TimeRangeQuery, parse_time_range};


mod context;
mod rows;
mod summary;
mod view;

use context::PerfTracesPageContext;

const BASE_URL: &str = "/admin/traces";
const PAGE_SIZE: i64 = 50;

#[derive(Debug, Deserialize)]
pub(crate) struct TraceListQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub preset: Option<String>,
    pub user_id: Option<UserId>,
    pub agent_id: Option<AgentId>,
    pub agent_scope: Option<String>,
    pub policy: Option<String>,
    pub decision: Option<String>,
    pub error_only: Option<String>,
    pub deny_only: Option<String>,
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub page: Option<i64>,
    pub group: Option<String>,
    pub project: Option<String>,
}

pub(crate) async fn perf_traces_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<TraceListQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let range = parse_time_range(&TimeRangeQuery {
        from: query.from.clone(),
        to: query.to.clone(),
        preset: query.preset.clone(),
    });
    let page = query.page.unwrap_or(0).max(0);

    let request =
        ScopeRequest::from_query(&user_ctx, query.group.as_deref(), query.project.as_deref());
    let scope = crate::repositories::scope::membership::get_subject_scope(&pool, &request).await?;
    let ctx = load_traces_data(
        &pool,
        &user_ctx,
        TraceScope {
            request: &request,
            subjects: scope,
        },
        &query,
        TraceWindow { range, page },
    )
    .await;
    Ok(super::render_typed_page(
        &engine,
        "perf-traces",
        &ctx,
        &user_ctx,
        &mkt_ctx,
    ))
}

// Why: the resolved window and page travel together — clippy's argument cap
// is the only reason they are not two parameters.
struct TraceWindow {
    range: TimeRange,
    page: i64,
}

// Why: the resolved user set and the request that produced it travel together —
// the queries bind the first, the filter form re-renders the second.
struct TraceScope<'a> {
    request: &'a ScopeRequest,
    subjects: SubjectScope,
}

#[expect(
    clippy::too_many_lines,
    reason = "one page assembly per handler; splitting is tracked in docs/tech-debt.md"
)]
async fn load_traces_data(
    pool: &PgPool,
    user_ctx: &UserContext,
    scope: TraceScope<'_>,
    query: &TraceListQuery,
    window: TraceWindow,
) -> PerfTracesPageContext {
    let TraceWindow { range, page } = window;
    let preset = preset_str(query, range);
    let filter = build_filter(query, scope.subjects.as_sql());
    let sort = sort_from_query(query);
    let offset = page * PAGE_SIZE;
    let trace_page = TracePage {
        sort,
        limit: PAGE_SIZE,
        offset,
    };
    let (list_res, stats_res, options_res) = tokio::join!(
        list_traces(pool, filter, range, trace_page),
        get_trace_stats(pool, range, scope.subjects.as_sql()),
        get_filter_options(pool, range),
    );

    let (rows, total) = list_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "list_traces failed");
        (Vec::new(), 0)
    });
    let stats = stats_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "get_trace_stats failed");
        TraceStats::default()
    });
    let options = options_res.unwrap_or_default();

    let trace_rows: Vec<rows::TraceRow> = rows.iter().map(rows::trace_to_json).collect();
    let has_traces = !trace_rows.is_empty();
    let window = list_view::PageWindow::new(
        page,
        PAGE_SIZE,
        total,
        i64::try_from(trace_rows.len()).unwrap_or(PAGE_SIZE),
        "traces",
    );
    let pagination = view::build_pagination(query, window);
    let sort_col = view::sort_col_to_str(sort.column);
    let sort_dir = view::sort_dir_to_str(sort.dir);

    PerfTracesPageContext {
        page: "traces",
        title: "Trace Explorer",
        breadcrumbs: vec![
            crate::handlers::ssr::types::BreadcrumbView::link("AI activity", "/admin/analytics"),
            crate::handlers::ssr::types::BreadcrumbView::current("Traces"),
        ],
        time_range: view::time_range_context(range, &preset),
        filter_ribbon: context::TraceFilterRibbon {
            base_url: BASE_URL,
            preserved: view::build_preserved(query, range, &preset),
            options: view::annotate_options(&options, &filter),
            chips: view::build_chips(query),
        },
        stats: summary::serde_stats(query, &stats),
        traces: trace_rows,
        has_traces,
        total_count: total,
        page_size: PAGE_SIZE,
        page_index: page,
        page_count: window.total_pages,
        pagination,
        sort_headers: summary::build_sort_headers(query, sort_col, sort_dir),
        sort: sort_col,
        dir: sort_dir,
        error_only: filter.error_only,
        deny_only: filter.deny_only,
        scope_filter: view::scope_filter(
            pool,
            user_ctx,
            &view::TraceScopeFilterArgs {
                request: scope.request,
                query,
                range,
                preset: &preset,
            },
        )
        .await,
    }
}

// Why: the query string is the only source of every filter column, so the
// mapping lives in one place rather than inline in the page assembly.
fn build_filter<'a>(
    query: &'a TraceListQuery,
    subject_ids: Option<&'a [String]>,
) -> TraceFilter<'a> {
    TraceFilter {
        subject_ids,
        user_id: empty_to_none(query.user_id.as_ref().map(UserId::as_str)),
        agent_id: empty_to_none(query.agent_id.as_ref().map(AgentId::as_str)),
        agent_scope: empty_to_none(query.agent_scope.as_deref()),
        policy: empty_to_none(query.policy.as_deref()),
        decision: empty_to_none(query.decision.as_deref()),
        error_only: query.error_only.as_deref() == Some("true"),
        deny_only: query.deny_only.as_deref() == Some("true"),
    }
}

fn empty_to_none(v: Option<&str>) -> Option<&str> {
    v.filter(|s| !s.is_empty())
}

fn preset_str(query: &TraceListQuery, range: TimeRange) -> String {
    if let Some(p) = query.preset.as_deref()
        && !p.is_empty()
    {
        return p.to_owned();
    }
    if query.from.is_some() && query.to.is_some() {
        return "custom".to_owned();
    }
    match range.preset {
        TimeRangePreset::Min15 => "15m",
        TimeRangePreset::Hour1 => "1h",
        TimeRangePreset::Hours24 => "24h",
        TimeRangePreset::Days7 => "7d",
        TimeRangePreset::Days30 => "30d",
        TimeRangePreset::Custom => "custom",
    }
    .to_owned()
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
