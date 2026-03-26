use serde_json::json;

pub(crate) fn compute_hourly_chart(
    hourly_activity: &[crate::admin::types::HourlyActivity],
) -> Vec<serde_json::Value> {
    let hourly_max = hourly_activity
        .iter()
        .map(|h| h.count)
        .max()
        .unwrap_or(1)
        .max(1);
    let mut hours = [0i64; 24];
    for h in hourly_activity {
        if let Ok(idx) = usize::try_from(h.hour) {
            if idx < 24 {
                hours[idx] = h.count;
            }
        }
    }
    hours.iter().enumerate().map(|(i, &count)| {
        let pct = count.saturating_mul(100) / hourly_max;
        json!({ "hour": i, "count": count, "pct": pct, "label": if i % 3 == 0 { format!("{i}") } else { String::new() } })
    }).collect()
}

pub(crate) fn compute_bar_chart<T, F, G>(
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

pub(crate) fn compute_area_chart_data(
    buckets: &[crate::admin::types::TimeSeriesBucket],
    range: &str,
) -> serde_json::Value {
    if buckets.is_empty() {
        return json!({
            "has_data": false,
            "paths": {},
            "x_labels": [],
            "y_labels": [],
            "peak": 0,
        });
    }

    let svg_w: f64 = 960.0;
    let svg_h: f64 = 280.0;
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
    paths.insert("top_line".to_string(), json!(top_line));

    let x_labels = build_x_labels(buckets, n, range);
    let y_labels = build_y_labels(peak, svg_h, y_max);

    json!({
        "has_data": true,
        "paths": paths,
        "x_labels": x_labels,
        "y_labels": y_labels,
        "peak": peak,
    })
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
        base.clone_from(cum.last().expect("cum is non-empty after push"));
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
) -> serde_json::Map<String, serde_json::Value> {
    use std::fmt::Write as _;
    let mut paths = serde_json::Map::new();
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
        paths.insert(name.to_string(), json!(d));
        prev_base.clone_from(top);
    }
    paths
}

fn build_svg_line(base: &[f64], n: usize, svg_w: f64, svg_h: f64, y_max: f64) -> String {
    use std::fmt::Write as _;
    let mut line = String::new();
    for (j, &total) in base.iter().enumerate() {
        let x = svg_x(j, n, svg_w);
        let y = svg_h - (total / y_max * svg_h);
        if j == 0 {
            let _ = write!(line, "M{x:.1},{y:.1}");
        } else {
            let _ = write!(line, " L{x:.1},{y:.1}");
        }
    }
    line
}

fn build_x_labels(
    buckets: &[crate::admin::types::TimeSeriesBucket],
    n: usize,
    range: &str,
) -> Vec<String> {
    let x_fmt = match range {
        "24h" => "%H:%M",
        "14d" => "%b %d",
        _ => "%a %H:%M",
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

fn build_y_labels(peak: i64, svg_h: f64, y_max: f64) -> Vec<serde_json::Value> {
    let y_step = (peak / 4).max(1);
    (0..=4i64)
        .map(|i| {
            let label = i * y_step;
            let val = f64::from(i32::try_from(label).unwrap_or(0));
            let y = svg_h - (val / y_max * svg_h);
            json!({ "label": label, "y": format!("{y:.1}") })
        })
        .collect()
}

fn svg_x(j: usize, n: usize, svg_w: f64) -> f64 {
    if n > 1 {
        f64::from(u32::try_from(j).unwrap_or(0)) / f64::from(u32::try_from(n - 1).unwrap_or(1))
            * svg_w
    } else {
        svg_w / 2.0
    }
}
