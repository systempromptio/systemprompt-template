use std::collections::HashMap;

use serde::Serialize;

use super::svg::{build_svg_line, build_y_labels, svg_x, AxisLabel, SVG_HEIGHT, SVG_WIDTH};

pub fn compute_bar_chart<T, F, G>(
    items: &[T],
    get_count: F,
    build_json: G,
) -> Vec<serde_json::Value>
where
    F: Fn(&T) -> i64,
    G: Fn(&T, i64) -> serde_json::Value,
{
    let max_val = items.first().map_or(1, &get_count).max(1);
    items
        .iter()
        .map(|item| {
            let pct = get_count(item).saturating_mul(100) / max_val;
            build_json(item, pct)
        })
        .collect()
}

#[derive(Serialize)]
pub struct AreaChart {
    has_data: bool,
    paths: HashMap<String, String>,
    x_labels: Vec<String>,
    y_labels: Vec<AxisLabel>,
    peak: i64,
}

pub fn compute_area_chart_data(
    buckets: &[crate::admin::types::TimeSeriesBucket],
    range: &str,
) -> AreaChart {
    if buckets.is_empty() {
        return AreaChart {
            has_data: false,
            paths: HashMap::new(),
            x_labels: Vec::new(),
            y_labels: Vec::new(),
            peak: 0,
        };
    }

    let svg_w: f64 = SVG_WIDTH;
    let svg_h: f64 = SVG_HEIGHT;
    let n = buckets.len();

    let peak = buckets
        .iter()
        .map(|b| b.sessions + b.active_users + b.prompts + b.tool_uses + b.errors)
        .max()
        .unwrap_or(1);
    let y_max = (f64::from(i32::try_from(peak).unwrap_or(i32::MAX)) * 1.1).max(1.0);

    let series_order: &[&str] = &["sessions", "active_users", "prompts", "tools", "errors"];
    let (cum, base) = compute_cumulative_series(buckets, series_order, n);
    let mut paths = build_area_paths(&cum, series_order, n, svg_w, svg_h, y_max);

    let top_line = build_svg_line(&base, n, svg_w, svg_h, y_max);
    paths.insert("top_line".to_string(), top_line);

    let x_labels = build_x_labels(buckets, n, range);
    let y_labels = build_y_labels(peak, svg_h, y_max);

    AreaChart {
        has_data: true,
        paths,
        x_labels,
        y_labels,
        peak,
    }
}

fn bucket_series_value(b: &crate::admin::types::TimeSeriesBucket, name: &str) -> f64 {
    let raw = match name {
        "sessions" => b.sessions,
        "active_users" => b.active_users,
        "prompts" => b.prompts,
        "tools" => b.tool_uses,
        "errors" => b.errors,
        _ => 0,
    };
    f64::from(i32::try_from(raw).unwrap_or(0))
}

fn compute_cumulative_series(
    buckets: &[crate::admin::types::TimeSeriesBucket],
    series_order: &[&str],
    n: usize,
) -> (Vec<Vec<f64>>, Vec<f64>) {
    let mut cum: Vec<Vec<f64>> = Vec::new();
    let mut base = vec![0.0f64; n];
    for &name in series_order {
        let top: Vec<f64> = buckets
            .iter()
            .zip(base.iter())
            .map(|(b, &prev)| prev + bucket_series_value(b, name))
            .collect();
        cum.push(top.clone());
        if let Some(last) = cum.last() {
            base.clone_from(last);
        }
    }
    (cum, base)
}

fn build_area_paths(
    cum: &[Vec<f64>],
    series_order: &[&str],
    n: usize,
    svg_w: f64,
    svg_h: f64,
    y_max: f64,
) -> HashMap<String, String> {
    use std::fmt::Write as _;
    let mut paths = HashMap::new();
    let mut prev_base = vec![0.0f64; n];

    for (i, &name) in series_order.iter().enumerate() {
        let top = &cum[i];
        let mut d = String::new();
        for (j, &y_val) in top.iter().enumerate() {
            let x = svg_x(j, n, svg_w);
            let y = svg_h - (y_val / y_max * svg_h);
            if j == 0 {
                let _ = write!(d, "M{x:.1},{y:.1}");
            } else {
                let _ = write!(d, " L{x:.1},{y:.1}");
            }
        }
        for j in (0..n).rev() {
            let x = svg_x(j, n, svg_w);
            let y = svg_h - (prev_base[j] / y_max * svg_h);
            let _ = write!(d, " L{x:.1},{y:.1}");
        }
        d.push('Z');
        paths.insert(name.to_string(), d);
        prev_base.clone_from(top);
    }
    paths
}

fn build_x_labels(
    buckets: &[crate::admin::types::TimeSeriesBucket],
    n: usize,
    range: &str,
) -> Vec<String> {
    let x_fmt = match range {
        "24h" => "%H:%M",
        "14d" => "%b %d",
        _ => "%a %d",
    };
    let step = (n / 7).max(1);
    buckets
        .iter()
        .enumerate()
        .filter_map(|(i, b)| {
            if i % step == 0 || i == n - 1 {
                Some(b.bucket.format(x_fmt).to_string())
            } else {
                None
            }
        })
        .collect()
}
