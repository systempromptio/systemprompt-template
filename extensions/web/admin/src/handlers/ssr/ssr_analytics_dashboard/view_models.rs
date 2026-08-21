//! The stacked cost-by-model chart: the one view that combines the model mix
//! with the cost series, so a spend spike can be attributed to a model without
//! reading two charts side by side.
//!
//! Split from `view.rs` at the 300-line ceiling.

use crate::handlers::ssr::format::format_cost;
use crate::handlers::ssr::types::{
    PieView, StackSeriesInput, StackedChartSpec, SvgLineChartView, SvgStackedChartView,
    stacked_chart,
};
use crate::repositories::analytics::site::model_series::ModelCostBucket;
use crate::repositories::analytics::site::series::UsageBucket;
use crate::util::time_range::TimeRange;

use super::view::{self, bucket_label, date_label, midpoint};

// Why: pivots the sparse per-model rows onto the spine the usage series
// already established, so the stacked bars line up bucket-for-bucket with the
// volume and cost lines instead of inventing a second calendar.
pub(super) fn model_cost_stack(
    rows: &[ModelCostBucket],
    spine: &[UsageBucket],
    range: &TimeRange,
    weekly: bool,
) -> SvgStackedChartView {
    let bucket_labels: Vec<String> = spine
        .iter()
        .map(|b| bucket_label(b.bucket_start, weekly))
        .collect();
    let index_of =
        |ts: chrono::DateTime<chrono::Utc>| spine.iter().position(|b| b.bucket_start == ts);

    let mut models: Vec<String> = Vec::new();
    let mut totals: Vec<i64> = Vec::new();
    for row in rows {
        if let Some(i) = models.iter().position(|m| *m == row.model) {
            totals[i] += row.cost_microdollars;
        } else {
            models.push(row.model.clone());
            totals.push(row.cost_microdollars);
        }
    }
    // Why: ranked descending so the color order matches the pie's for the
    // same window; "Other" sorts by its own total like any other series.
    let mut order: Vec<usize> = (0..models.len()).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(totals[i]));

    let series = order
        .iter()
        .map(|&i| {
            let mut values = vec![0i64; spine.len()];
            for row in rows.iter().filter(|r| r.model == models[i]) {
                if let Some(b) = index_of(row.bucket_start) {
                    values[b] += row.cost_microdollars;
                }
            }
            StackSeriesInput {
                label: models[i].clone(),
                values,
                value_display: format_cost(totals[i]),
            }
        })
        .collect();

    let grand_total: i64 = totals.iter().sum();
    stacked_chart(StackedChartSpec {
        title: "Cost by model",
        subtitle: format!(
            "{} across {} models",
            format_cost(grand_total),
            models.len()
        ),
        empty_message: "No billed requests in this window.",
        series,
        bucket_labels,
        value_display: format_cost,
        x_start_display: date_label(range.from),
        x_mid_display: date_label(midpoint(range)),
        x_end_display: date_label(range.to),
    })
}

// Why: the four Overview visuals are built together because they share one
// window and one bucket spine — keeping them in one place is what stops the
// stacked cost chart from drifting onto a different calendar than the lines.
pub(super) struct OverviewCharts {
    pub volume: SvgLineChartView,
    pub cost: SvgLineChartView,
    pub model_pie: PieView,
    pub model_cost: SvgStackedChartView,
}

pub(super) fn overview_charts(
    fetched: &super::data::AnalyticsDashboardData,
    query: &super::AnalyticsDashboardQuery,
    range: &TimeRange,
    weekly: bool,
) -> OverviewCharts {
    OverviewCharts {
        volume: view::volume_chart(&fetched.series, range, weekly),
        cost: view::spend_chart(&fetched.series, range, weekly),
        model_pie: view::model_pie(&fetched.models, query),
        model_cost: model_cost_stack(&fetched.model_cost, &fetched.series, range, weekly),
    }
}
