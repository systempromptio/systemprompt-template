//! The thirty-day request chart on the group detail page.

use super::format::short_num;
use super::types::{LineChartSpec, SvgLineChartView, SvgSeriesInput, line_chart};

pub(crate) fn daily_requests_chart(daily: &[i64]) -> SvgLineChartView {
    let total: i64 = daily.iter().sum();
    line_chart(LineChartSpec {
        // Why: the caption says what the series is, not what the section is.
        // The section heading above it already carries "Requests, last 30
        // days", and repeating that here read as a stutter on the page.
        title: "Daily volume",
        subtitle: format!("{} requests", short_num(total)),
        empty_message: "No gateway traffic in the last 30 days.",
        series: vec![SvgSeriesInput {
            label: "Requests".to_owned(),
            values: daily.to_vec(),
            value_display: short_num(total),
        }],
        ref_lines: Vec::new(),
        y_max: None,
        y_display: |v| v.to_string(),
        x_start_display: "30d ago".to_owned(),
        x_mid_display: "15d ago".to_owned(),
        x_end_display: "today".to_owned(),
        show_area: true,
    })
}
