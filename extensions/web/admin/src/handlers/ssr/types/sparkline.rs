//! Sparkline and delta view-models for KPI tiles and table rows.
//!
//! Same rule as the charts: geometry is computed here from
//! [`crate::util::svg`], the partial only prints attributes.

use serde::Serialize;

use crate::util::delta::{Delta, delta};
use crate::util::svg;

// Why: decoration only (`aria-hidden`) — the card's value and delta text are
// the accessible copy. Its own 100x24 unit space, not the chart's 100x40.
// `tone` colours the stroke; the last point is marked so the eye lands on
// "now"; `title` is the hover text a table cell offers.
#[derive(Debug, Serialize)]
pub(crate) struct SparklineView {
    pub path_d: String,
    pub area_d: String,
    pub has_data: bool,
    pub tone: &'static str,
    pub last_x: String,
    pub last_y: String,
    pub title: String,
}

pub(crate) fn sparkline(values: &[i64]) -> SparklineView {
    sparkline_toned(values, "accent", String::new())
}

pub(crate) fn sparkline_toned(values: &[i64], tone: &'static str, title: String) -> SparklineView {
    let max = values.iter().copied().max().unwrap_or(0);
    if max <= 0 || values.len() < 2 {
        return SparklineView {
            path_d: String::new(),
            area_d: String::new(),
            has_data: false,
            tone,
            last_x: String::new(),
            last_y: String::new(),
            title,
        };
    }
    let points: Vec<(f64, f64)> = svg::scale_points(values, max)
        .into_iter()
        .map(|(x, y)| (x, y / svg::PLOT_H * 24.0))
        .collect();
    let (last_x, last_y) = points.last().copied().unwrap_or((0.0, 0.0));
    // Why: the area closes on the 24-unit baseline, not the chart's 40.
    let area_d = format!(
        "{} L{:.2},24 L{:.2},24 Z",
        svg::line_path(&points),
        last_x,
        points.first().map_or(0.0, |p| p.0)
    );
    SparklineView {
        path_d: svg::line_path(&points),
        area_d,
        has_data: true,
        tone,
        last_x: format!("{last_x:.2}"),
        last_y: format!("{last_y:.2}"),
        title,
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
