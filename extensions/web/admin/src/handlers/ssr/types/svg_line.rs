//! Server-rendered SVG line-chart and sparkline view-models.
//!
//! All geometry comes from [`crate::util::svg`]; this module only shapes it
//! for the `svg-line-chart` / `kpi-sparkline` partials. Same design rule as
//! `charts.rs`: the view carries every derived value (paths, axis labels,
//! reference-line positions), so the template does no arithmetic and the axis
//! cannot disagree with the lines.

use serde::Serialize;

use crate::util::delta::{Delta, delta};
use crate::util::svg;

// Why: shared with the pie so a model keeps one color across every chart in
// a window.
pub(crate) const CHART_COLOR_TOKENS: [&str; 7] = [
    "--sp-chart-purple",
    "--sp-chart-blue",
    "--sp-chart-green",
    "--sp-chart-amber",
    "--sp-chart-red",
    "--sp-chart-cyan",
    "--sp-chart-indigo",
];

#[derive(Debug, Serialize)]
pub(crate) struct SvgLineChartView {
    // Why: serialized as chart_title — the layout partial's `title=` hash
    // param shadows a context field named `title` inside nested partials.
    #[serde(rename = "chart_title")]
    pub title: &'static str,
    pub subtitle: String,
    pub has_data: bool,
    pub empty_message: &'static str,
    pub aria_label: String,
    pub series: Vec<SvgSeriesView>,
    pub ref_lines: Vec<SvgRefLineView>,
    pub legend: Vec<SvgLegendItemView>,
    pub y_max_display: String,
    pub y_mid_display: String,
    pub x_start_display: String,
    pub x_mid_display: String,
    pub x_end_display: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct SvgSeriesView {
    pub path_d: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub area_d: Option<String>,
    pub color_token: &'static str,
    pub label: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct SvgRefLineView {
    pub y: String,
    pub label: String,
    pub tone: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct SvgLegendItemView {
    pub label: String,
    pub color_index: usize,
    pub value_display: String,
}

// Why: one series' inputs before geometry — a label, its values already
// bucketed onto the shared spine, and the total the legend prints.
pub(crate) struct SvgSeriesInput {
    pub label: String,
    pub values: Vec<i64>,
    pub value_display: String,
}

// Why: `y_max` of `None` derives a round max from the data; `Some` pins it —
// the burn-up pins to the cap so the cap line always has room on the plot.
pub(crate) struct LineChartSpec {
    pub title: &'static str,
    pub subtitle: String,
    pub empty_message: &'static str,
    pub series: Vec<SvgSeriesInput>,
    pub ref_lines: Vec<(i64, String, &'static str)>,
    pub y_max: Option<i64>,
    pub y_display: fn(i64) -> String,
    pub x_start_display: String,
    pub x_mid_display: String,
    pub x_end_display: String,
    // Why: Fill under the line — single-series charts only (stacked fills lie).
    pub show_area: bool,
}

pub(crate) fn line_chart(spec: LineChartSpec) -> SvgLineChartView {
    let data_max = spec
        .series
        .iter()
        .flat_map(|s| s.values.iter().copied())
        .max()
        .unwrap_or(0);
    let y_max = spec
        .y_max
        .map_or_else(|| svg::nice_max(data_max), |m| m.max(1));
    let has_data = data_max > 0;
    let multi = spec.series.len() > 1;

    let series: Vec<SvgSeriesView> = spec
        .series
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let points = svg::scale_points(&s.values, y_max);
            SvgSeriesView {
                path_d: svg::line_path(&points),
                area_d: (spec.show_area && !multi).then(|| svg::area_path(&points)),
                color_token: CHART_COLOR_TOKENS[i.min(CHART_COLOR_TOKENS.len() - 1)],
                label: s.label.clone(),
            }
        })
        .collect();

    let ref_lines = spec
        .ref_lines
        .iter()
        .filter_map(|(value, label, tone)| {
            svg::ref_line_y(*value, y_max).map(|y| SvgRefLineView {
                y: format!("{y:.2}"),
                label: label.clone(),
                tone,
            })
        })
        .collect();

    let legend: Vec<SvgLegendItemView> = if multi {
        spec.series
            .iter()
            .enumerate()
            .map(|(i, s)| SvgLegendItemView {
                label: s.label.clone(),
                color_index: i.min(CHART_COLOR_TOKENS.len() - 1) + 1,
                value_display: s.value_display.clone(),
            })
            .collect()
    } else {
        Vec::new()
    };

    let aria_label = format!(
        "{}: {} — peak {}",
        spec.title,
        spec.series
            .iter()
            .map(|s| format!("{} {}", s.label, s.value_display))
            .collect::<Vec<_>>()
            .join(", "),
        (spec.y_display)(data_max)
    );

    SvgLineChartView {
        title: spec.title,
        subtitle: spec.subtitle,
        has_data,
        empty_message: spec.empty_message,
        aria_label,
        series,
        ref_lines,
        legend,
        y_max_display: (spec.y_display)(y_max),
        y_mid_display: (spec.y_display)(y_max / 2),
        x_start_display: spec.x_start_display,
        x_mid_display: spec.x_mid_display,
        x_end_display: spec.x_end_display,
    }
}

// Why: decoration only (`aria-hidden`) — the card's value and delta text are
// the accessible copy. Its own 100x24 unit space, not the chart's 100x40.
#[derive(Debug, Serialize)]
pub(crate) struct SparklineView {
    pub path_d: String,
    pub has_data: bool,
}

pub(crate) fn sparkline(values: &[i64]) -> SparklineView {
    let max = values.iter().copied().max().unwrap_or(0);
    if max <= 0 {
        return SparklineView {
            path_d: String::new(),
            has_data: false,
        };
    }
    let points: Vec<(f64, f64)> = svg::scale_points(values, max)
        .into_iter()
        .map(|(x, y)| (x, y / svg::PLOT_H * 24.0))
        .collect();
    SparklineView {
        path_d: svg::line_path(&points),
        has_data: true,
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct DeltaView {
    pub display: String,
    pub direction: &'static str,
    pub tone: &'static str,
}

pub(crate) fn delta_view(current: i64, previous: i64, up_is_good: bool) -> DeltaView {
    let d: Delta = delta(current, previous, up_is_good);
    DeltaView {
        display: d.display(),
        direction: d.direction,
        tone: d.tone,
    }
}
