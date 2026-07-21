//! Data-collection orchestration for the Inference Requests page.
//!
//! Resolves the effective time range (honouring an explicit user pick, else
//! auto-widening 24h -> 7d -> 30d until a window has rows) and runs the five
//! parallel repository fetches, collapsing each `Result` into a logged default
//! so a single failed query never takes the whole page down.

use std::sync::Arc;

use sqlx::PgPool;

use crate::repositories::analytics::request_stats::{
    CostBucket, LatencyBucket, RequestStats, get_request_stats, list_cost_over_time,
    list_latency_histogram,
};
use crate::repositories::analytics::requests::{
    RequestFilter, RequestFilterOptions, RequestPage, RequestRow, RequestSortSpec,
    get_request_filter_options, list_requests_paged,
};
use crate::util::time_range::{
    TimeRange, TimeRangePreset, TimeRangeQuery, count_requests_in_range, parse_time_range,
    preset_to_range,
};

use super::RequestsQuery;

pub(super) async fn resolve_range(
    pool: &PgPool,
    query: &RequestsQuery,
) -> (TimeRange, Option<&'static str>) {
    let user_picked_range = query.preset.is_some() || (query.from.is_some() && query.to.is_some());
    let initial_range = parse_time_range(&TimeRangeQuery {
        from: query.from.clone(),
        to: query.to.clone(),
        preset: query.preset.clone(),
    });

    if user_picked_range {
        return (initial_range, None);
    }

    let mut chosen = initial_range;
    let mut widened: Option<&'static str> = None;
    for (label, preset) in [
        ("24h", TimeRangePreset::Hours24),
        ("7d", TimeRangePreset::Days7),
        ("30d", TimeRangePreset::Days30),
    ] {
        let candidate = preset_to_range(preset);
        let count = count_requests_in_range(pool, candidate).await.unwrap_or(0);
        if count > 0 {
            chosen = candidate;
            widened = if label == "24h" { None } else { Some(label) };
            break;
        }
    }
    (chosen, widened)
}

pub(super) struct RequestsData {
    pub rows: Vec<RequestRow>,
    pub total_count: i64,
    pub stats: RequestStats,
    pub hist: Vec<LatencyBucket>,
    pub cost: Vec<CostBucket>,
    pub options: RequestFilterOptions,
}

pub(super) struct RequestsPageQuery<'a> {
    pub filter: &'a RequestFilter,
    pub range: TimeRange,
    pub sort: RequestSortSpec,
    pub page_size: i64,
    pub offset: i64,
}

pub(super) async fn fetch_requests_data(
    pool: &Arc<PgPool>,
    query: RequestsPageQuery<'_>,
) -> RequestsData {
    let RequestsPageQuery {
        filter,
        range,
        sort,
        page_size,
        offset,
    } = query;
    let page = RequestPage {
        sort,
        limit: page_size,
        offset,
    };
    let (paged, stats_res, hist_res, cost_res, options_res) = tokio::join!(
        list_requests_paged(pool, filter, range, page),
        get_request_stats(pool, range),
        list_latency_histogram(pool, range),
        list_cost_over_time(pool, range),
        get_request_filter_options(pool, range),
    );

    let (rows, total_count) = paged.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "list_requests_paged failed");
        (Vec::new(), 0)
    });
    let stats = stats_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "get_request_stats failed");
        RequestStats::default()
    });
    let hist = hist_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "list_latency_histogram failed");
        Vec::new()
    });
    let cost = cost_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "list_cost_over_time failed");
        Vec::new()
    });
    let options = options_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "get_request_filter_options failed");
        RequestFilterOptions::default()
    });

    RequestsData {
        rows,
        total_count,
        stats,
        hist,
        cost,
        options,
    }
}
