//! Server-rendered stacked-bar view-model (cost by model over time).
//!
//! The template loops one flat `rects` list — every rectangle arrives with
//! preformatted plot-space coordinates and a tooltip — so all stacking math
//! lives here. Series are ranked descending by total and colored with the
//! same token order as the pie, so model→color agrees across a window.

use serde::Serialize;

use crate::util::svg;

use super::svg_line::{CHART_COLOR_TOKENS, SvgLegendItemView};

#[derive(Debug, Serialize)]
pub(crate) struct SvgStackedChartView {
    // Why: serialized as chart_title — the layout partial's `title=` hash
    // param shadows a context field named `title` inside nested partials.
    #[serde(rename = "chart_title")]
    pub title: &'static str,
    pub subtitle: String,
    pub has_data: bool,
    pub empty_message: &'static str,
    pub aria_label: String,
    pub rects: Vec<SvgRectView>,
    pub legend: Vec<SvgLegendItemView>,
    pub y_max_display: String,
    pub y_mid_display: String,
    pub x_start_display: String,
    pub x_mid_display: String,
    pub x_end_display: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct SvgRectView {
    pub x: String,
    pub y: String,
    pub w: String,
    pub h: String,
    pub color_token: &'static str,
    pub tooltip: String,
}

// Why: one series' inputs, already ranked by the caller so color order
// matches the pie's for the same window.
pub(crate) struct StackSeriesInput {
    pub label: String,
    pub values: Vec<i64>,
    pub value_display: String,
}

pub(crate) struct StackedChartSpec {
    pub title: &'static str,
    pub subtitle: String,
    pub empty_message: &'static str,
    pub series: Vec<StackSeriesInput>,
    pub bucket_labels: Vec<String>,
    pub value_display: fn(i64) -> String,
    pub x_start_display: String,
    pub x_mid_display: String,
    pub x_end_display: String,
}

pub(crate) fn stacked_chart(spec: StackedChartSpec) -> SvgStackedChartView {
    let buckets = spec.bucket_labels.len();
    let totals: Vec<i64> = (0..buckets)
        .map(|b| {
            spec.series
                .iter()
                .map(|s| s.values.get(b).copied().unwrap_or(0))
                .sum()
        })
        .collect();
    let raw_max = totals.iter().copied().max().unwrap_or(0);
    let y_max = svg::nice_max(raw_max);
    let has_data = raw_max > 0;

    let slots = svg::bar_slots(buckets, 0.25);
    let mut rects = Vec::new();
    for (b, &(x, w)) in slots.iter().enumerate() {
        let values: Vec<i64> = spec
            .series
            .iter()
            .map(|s| s.values.get(b).copied().unwrap_or(0))
            .collect();
        let segments = svg::stack_segments(&values, y_max);
        for (si, (&value, &(y, h))) in values.iter().zip(segments.iter()).enumerate() {
            if value <= 0 {
                continue;
            }
            rects.push(SvgRectView {
                x: format!("{x:.2}"),
                y: format!("{y:.2}"),
                w: format!("{w:.2}"),
                h: format!("{h:.2}"),
                color_token: CHART_COLOR_TOKENS[si.min(CHART_COLOR_TOKENS.len() - 1)],
                tooltip: format!(
                    "{}: {} — {}",
                    spec.bucket_labels[b],
                    spec.series[si].label,
                    (spec.value_display)(value)
                ),
            });
        }
    }

    let legend: Vec<SvgLegendItemView> = spec
        .series
        .iter()
        .enumerate()
        .map(|(i, s)| SvgLegendItemView {
            label: s.label.clone(),
            color_index: i.min(CHART_COLOR_TOKENS.len() - 1) + 1,
            value_display: s.value_display.clone(),
        })
        .collect();

    let aria_label = format!(
        "{}: {}",
        spec.title,
        legend
            .iter()
            .map(|l| format!("{} {}", l.label, l.value_display))
            .collect::<Vec<_>>()
            .join(", ")
    );

    SvgStackedChartView {
        title: spec.title,
        subtitle: spec.subtitle,
        has_data,
        empty_message: spec.empty_message,
        aria_label,
        rects,
        legend,
        y_max_display: (spec.value_display)(y_max),
        y_mid_display: (spec.value_display)(y_max / 2),
        x_start_display: spec.x_start_display,
        x_mid_display: spec.x_mid_display,
        x_end_display: spec.x_end_display,
    }
}
