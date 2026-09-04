//! Shared chart view-models for the admin analytics pages.
//!
//! Two shapes, both rendered by CSS-only partials (no client charting library):
//! [`HistogramView`] for the horizontal latency distribution and [`ChartView`]
//! for the vertical time series. Both carry their own axis labels, so the
//! template never has to derive a scale — a bar's `pct` is always relative to
//! the `max` printed on the axis, which is what makes the two agree.
//!
//! The Inference Requests and Evals pages both build these, so a fix to how a
//! chart reads lands on both at once.

use serde::Serialize;

use crate::handlers::ssr::format::{format_cost, format_duration_ms};
use crate::repositories::analytics::request_stats::{LatencyBucket, RequestStats, TimeBucket};
use crate::util::time_range::TimeRange;

// Why: the view carries its own axis labels — y_max/y_mid label the gridlines
// the partial draws, x_* label the window the buckets span — so the template
// never derives a scale and the axis cannot disagree with the bars.
#[derive(Debug, Serialize)]
pub(crate) struct ChartView {
    pub heading: &'static str,
    pub subtitle: String,
    pub tone: &'static str,
    pub series: Vec<ChartBarView>,
    pub has_data: bool,
    pub y_max_display: String,
    pub y_mid_display: String,
    pub x_start_display: String,
    pub x_mid_display: String,
    pub x_end_display: String,
    pub empty_message: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct ChartBarView {
    pub pct: i64,
    pub tooltip: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct HistogramView {
    pub bars: Vec<HistogramBarView>,
    pub max_display: String,
    pub has_data: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p50_display: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p95_display: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct HistogramBarView {
    pub label: String,
    pub count: i64,
    pub pct: i64,
    pub is_peak: bool,
}

// Why: bars scale to the busiest bin so the shape reads at a glance, and that
// bin is flagged so the partial can tint it. The p50 / p95 captions come from
// the same stats the KPI strip prints, so the two cannot disagree.
pub(crate) fn histogram_view(buckets: &[LatencyBucket], stats: &RequestStats) -> HistogramView {
    let max = buckets.iter().map(|b| b.count).max().unwrap_or(0);
    let has_data = buckets.iter().any(|b| b.count > 0);
    HistogramView {
        bars: buckets
            .iter()
            .map(|b| HistogramBarView {
                label: b.label.clone(),
                count: b.count,
                pct: bar_pct(b.count, max),
                is_peak: max > 0 && b.count == max,
            })
            .collect(),
        max_display: max.to_string(),
        has_data,
        p50_display: has_data.then(|| format_duration_ms(stats.p50_latency_ms.round() as i64)),
        p95_display: has_data.then(|| format_duration_ms(stats.p95_latency_ms.round() as i64)),
    }
}

pub(crate) fn traffic_chart(buckets: &[TimeBucket], range: &TimeRange) -> ChartView {
    let max = buckets.iter().map(|b| b.requests).max().unwrap_or(0);
    let total: i64 = buckets.iter().map(|b| b.requests).sum();
    let errors: i64 = buckets.iter().map(|b| b.errors).sum();
    ChartView {
        heading: "Requests over time",
        subtitle: format!("{total} calls · {errors} failed · peak {max} per bucket"),
        tone: "accent",
        series: buckets
            .iter()
            .map(|b| ChartBarView {
                pct: bar_pct(b.requests, max),
                tooltip: format!(
                    "{}: {} requests, {} failed",
                    format_bucket_time(&b.bucket_start),
                    b.requests,
                    b.errors
                ),
            })
            .collect(),
        has_data: max > 0,
        y_max_display: max.to_string(),
        y_mid_display: (max / 2).to_string(),
        x_start_display: format_bucket_time(&range.from),
        x_mid_display: format_bucket_time(&midpoint(range)),
        x_end_display: format_bucket_time(&range.to),
        empty_message: "No gateway calls in this window.",
    }
}

pub(crate) fn cost_chart(buckets: &[TimeBucket], range: &TimeRange) -> ChartView {
    let max = buckets
        .iter()
        .map(|b| b.cost_microdollars)
        .max()
        .unwrap_or(0);
    let total: i64 = buckets.iter().map(|b| b.cost_microdollars).sum();
    ChartView {
        heading: "Cost over time",
        subtitle: format!(
            "{} across the window · peak {} per bucket",
            format_cost(total),
            format_cost(max)
        ),
        tone: "success",
        series: buckets
            .iter()
            .map(|b| ChartBarView {
                pct: bar_pct(b.cost_microdollars, max),
                tooltip: format!(
                    "{}: {}",
                    format_bucket_time(&b.bucket_start),
                    format_cost(b.cost_microdollars)
                ),
            })
            .collect(),
        has_data: max > 0,
        y_max_display: format_cost(max),
        y_mid_display: format_cost(max / 2),
        x_start_display: format_bucket_time(&range.from),
        x_mid_display: format_bucket_time(&midpoint(range)),
        x_end_display: format_bucket_time(&range.to),
        empty_message: "No billed requests in this window.",
    }
}

// Why: any non-zero value floors at 2% so a single request in a bucket is still
// visible; an empty series yields 0. Shared with the breakdown share bars so
// every bar on the page scales by one rule.
pub(crate) fn bar_pct(value: i64, max: i64) -> i64 {
    if max <= 0 || value <= 0 {
        return 0;
    }
    let pct = (value as f64 / max as f64 * 100.0).round() as i64;
    pct.clamp(2, 100)
}

fn midpoint(range: &TimeRange) -> chrono::DateTime<chrono::Utc> {
    range.from + (range.to - range.from) / 2
}

fn format_bucket_time(ts: &chrono::DateTime<chrono::Utc>) -> String {
    ts.with_timezone(&chrono::Local)
        .format("%b %d %H:%M")
        .to_string()
}
