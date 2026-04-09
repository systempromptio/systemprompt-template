use serde::Serialize;

use super::svg::{
    build_stacked_area, build_svg_line, build_y_labels, svg_x, AxisLabel, XAxisLabel, SVG_HEIGHT,
    SVG_WIDTH,
};

#[derive(Serialize)]
struct TrafficTooltip {
    label: String,
    sessions: i64,
    page_views: i64,
    x: String,
}

#[derive(Serialize, Debug)]
pub struct TrafficChart {
    has_data: bool,
    area_path: String,
    line_path: String,
    x_labels: Vec<XAxisLabel>,
    y_labels: Vec<AxisLabel>,
    peak: i64,
    buckets_json: String,
}

pub fn compute_traffic_chart_data(
    buckets: &[crate::admin::types::TrafficTimeBucket],
    range: &str,
) -> TrafficChart {
    if buckets.is_empty() {
        return TrafficChart {
            has_data: false,
            area_path: String::new(),
            line_path: String::new(),
            x_labels: Vec::new(),
            y_labels: Vec::new(),
            peak: 0,
            buckets_json: "[]".to_string(),
        };
    }

    let svg_w: f64 = SVG_WIDTH;
    let svg_h: f64 = SVG_HEIGHT;
    let n = buckets.len();

    let peak = buckets.iter().map(|b| b.sessions).max().unwrap_or(1).max(1);
    let y_max = (f64::from(i32::try_from(peak).unwrap_or(i32::MAX)) * 1.1).max(1.0);

    let sessions: Vec<f64> = buckets
        .iter()
        .map(|b| f64::from(i32::try_from(b.sessions).unwrap_or(0)))
        .collect();

    let base = vec![0.0f64; n];
    let area_path = build_stacked_area(&sessions, &base, n, svg_w, svg_h, y_max);
    let line_path = build_svg_line(&sessions, n, svg_w, svg_h, y_max);

    let x_fmt = match range {
        "7d" => "%a %d",
        "30d" => "%b %d",
        _ => "%H:%M",
    };
    let step = (n / 7).max(1);
    let x_labels: Vec<XAxisLabel> = buckets
        .iter()
        .enumerate()
        .filter_map(|(i, b)| {
            if i % step == 0 || i == n - 1 {
                let x = svg_x(i, n, svg_w);
                Some(XAxisLabel {
                    label: b.bucket.format(x_fmt).to_string(),
                    x: format!("{x:.1}"),
                })
            } else {
                None
            }
        })
        .collect();

    let y_labels = build_y_labels(peak, svg_h, y_max);

    let tooltip_fmt = match range {
        "7d" | "30d" => "%b %d",
        _ => "%H:%M",
    };
    let tooltips: Vec<TrafficTooltip> = buckets
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let x = svg_x(i, n, svg_w);
            TrafficTooltip {
                label: b.bucket.format(tooltip_fmt).to_string(),
                sessions: b.sessions,
                page_views: b.page_views,
                x: format!("{x:.1}"),
            }
        })
        .collect();

    TrafficChart {
        has_data: true,
        area_path,
        line_path,
        x_labels,
        y_labels,
        peak,
        buckets_json: serde_json::to_string(&tooltips).unwrap_or_else(|_| String::new()),
    }
}
