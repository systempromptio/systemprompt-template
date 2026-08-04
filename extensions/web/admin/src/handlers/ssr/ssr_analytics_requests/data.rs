//! Data-collection orchestration for the Inference Requests page.
//!
//! Resolves the effective time range (honouring an explicit user pick, else
//! auto-widening 24h -> 7d -> 30d until a window has rows), then runs only the
//! queries the active tab actually renders. Every `Result` collapses into a
//! logged default so a single failed query never takes the whole page down.

use std::sync::Arc;

use sqlx::PgPool;

use crate::repositories::analytics::request_stats::{
    LatencyBucket, RequestStats, TimeBucket, get_request_stats, list_latency_histogram,
    list_request_timeseries,
};
use crate::repositories::analytics::requests::{
    BreakdownRow, RequestFilter, RequestPage, RequestRow, RequestSortSpec, list_requests_by_model,
    list_requests_by_provider, list_requests_by_status, list_requests_paged,
};
use crate::util::time_range::{
    TimeRange, TimeRangePreset, TimeRangeQuery, count_requests_in_range, parse_time_range,
    preset_to_range,
};

use super::RequestsQuery;
use super::context::RequestsTab;

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

#[derive(Default)]
pub(super) struct RequestsData {
    pub rows: Vec<RequestRow>,
    pub total_count: i64,
    pub stats: RequestStats,
    pub hist: Vec<LatencyBucket>,
    pub series: Vec<TimeBucket>,
    pub breakdown: Vec<BreakdownRow>,
}

pub(super) struct RequestsPageQuery<'a> {
    pub tab: RequestsTab,
    pub filter: &'a RequestFilter,
    pub range: TimeRange,
    pub sort: RequestSortSpec,
    pub page_size: i64,
    pub offset: i64,
}

// Why: the KPI strip and the Log tab's count render on every tab, so the paged
// list and the stats always run; the charts and the rollups only run for the
// tab that shows them.
pub(super) async fn load_requests_data(
    pool: &Arc<PgPool>,
    query: RequestsPageQuery<'_>,
) -> RequestsData {
    let RequestsPageQuery {
        tab,
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

    let (paged, stats_res) = tokio::join!(
        list_requests_paged(pool, filter, range, page),
        get_request_stats(pool, range),
    );

    let (rows, total_count) = paged.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "list_requests_paged failed");
        (Vec::new(), 0)
    });
    let stats = stats_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "get_request_stats failed");
        RequestStats::default()
    });

    let mut data = RequestsData {
        rows,
        total_count,
        stats,
        ..RequestsData::default()
    };

    match tab {
        RequestsTab::Overview => {
            let (hist_res, series_res) = tokio::join!(
                list_latency_histogram(pool, range),
                list_request_timeseries(pool, range),
            );
            data.hist = unwrap_or_empty(hist_res, "list_latency_histogram");
            data.series = unwrap_or_empty(series_res, "list_request_timeseries");
        },
        RequestsTab::Models => {
            data.breakdown = unwrap_or_empty(
                list_requests_by_model(pool, range).await,
                "list_requests_by_model",
            );
        },
        RequestsTab::Providers => {
            data.breakdown = unwrap_or_empty(
                list_requests_by_provider(pool, range).await,
                "list_requests_by_provider",
            );
        },
        RequestsTab::Status => {
            data.breakdown = unwrap_or_empty(
                list_requests_by_status(pool, range).await,
                "list_requests_by_status",
            );
        },
        RequestsTab::Log => {},
    }

    data
}

fn unwrap_or_empty<T>(res: Result<Vec<T>, sqlx::Error>, what: &'static str) -> Vec<T> {
    res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, query = what, "requests page query failed");
        Vec::new()
    })
}
