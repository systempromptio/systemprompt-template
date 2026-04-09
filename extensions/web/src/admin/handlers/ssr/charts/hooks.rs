use serde::Serialize;

use super::svg::{
    build_stacked_area, build_svg_line, build_y_labels, AxisLabel, SVG_HEIGHT, SVG_WIDTH,
};

#[derive(Debug, Serialize)]
struct HooksChartPaths {
    errors: String,
    events: String,
    top_line: String,
}

#[derive(Serialize, Debug)]
pub struct HooksChart {
    has_data: bool,
    paths: HooksChartPaths,
    x_labels: Vec<String>,
    y_labels: Vec<AxisLabel>,
    peak: i64,
}

pub fn compute_hooks_chart_data(
    buckets: &[crate::admin::types::HookTimeSeriesBucket],
    range: &str,
) -> HooksChart {
    if buckets.is_empty() {
        return HooksChart {
            has_data: false,
            paths: HooksChartPaths {
                errors: String::new(),
                events: String::new(),
                top_line: String::new(),
            },
            x_labels: Vec::new(),
            y_labels: Vec::new(),
            peak: 0,
        };
    }

    let svg_w: f64 = SVG_WIDTH;
    let svg_h: f64 = SVG_HEIGHT;
    let n = buckets.len();

    let peak = buckets.iter().map(|b| b.event_count).max().unwrap_or(1);
    let y_max = (f64::from(i32::try_from(peak).unwrap_or(i32::MAX)) * 1.1).max(1.0);

    let errors: Vec<f64> = buckets
        .iter()
        .map(|b| f64::from(i32::try_from(b.error_count).unwrap_or(0)))
        .collect();
    let error_path = build_stacked_area(&errors, &vec![0.0; n], n, svg_w, svg_h, y_max);

    let events_top: Vec<f64> = buckets
        .iter()
        .zip(errors.iter())
        .map(|(b, &err)| err + f64::from(i32::try_from(b.event_count).unwrap_or(0)))
        .collect();
    let events_path = build_stacked_area(&events_top, &errors, n, svg_w, svg_h, y_max);
    let top_line = build_svg_line(&events_top, n, svg_w, svg_h, y_max);

    let x_fmt = match range {
        "24h" => "%H:%M",
        "14d" => "%b %d",
        _ => "%a %d",
    };
    let step = (n / 7).max(1);
    let x_labels: Vec<String> = buckets
        .iter()
        .enumerate()
        .filter_map(|(i, b)| {
            if i % step == 0 || i == n - 1 {
                Some(b.bucket.format(x_fmt).to_string())
            } else {
                None
            }
        })
        .collect();

    let y_labels = build_y_labels(peak, svg_h, y_max);

    HooksChart {
        has_data: true,
        paths: HooksChartPaths {
            errors: error_path,
            events: events_path,
            top_line,
        },
        x_labels,
        y_labels,
        peak,
    }
}
