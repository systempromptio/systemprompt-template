use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::svg::{
    build_stacked_area, build_y_labels, svg_x, AxisLabel, XAxisLabel, SVG_HEIGHT, SVG_WIDTH,
};

const COUNTRY_COLORS: &[&str] = &[
    "oklch(0.70 0.15 145)",
    "oklch(0.65 0.15 250)",
    "oklch(0.70 0.15 30)",
    "oklch(0.65 0.15 300)",
    "oklch(0.70 0.15 180)",
    "oklch(0.65 0.15 350)",
    "oklch(0.75 0.12 90)",
    "oklch(0.60 0.15 220)",
    "oklch(0.70 0.10 50)",
    "oklch(0.65 0.12 160)",
];

const DEFAULT_COLOR: &str = "oklch(0.60 0.10 200)";

fn f64_to_i64(v: f64) -> i64 {
    let s = format!("{:.0}", v.round());
    s.parse::<i64>().unwrap_or(0)
}

fn country_color(i: usize) -> &'static str {
    COUNTRY_COLORS.get(i).copied().unwrap_or(DEFAULT_COLOR)
}

#[derive(Serialize)]
struct CountryArea {
    name: String,
    color: &'static str,
    area_path: String,
}

#[derive(Serialize)]
struct TooltipBreakdown {
    country: String,
    sessions: i64,
    color: &'static str,
}

#[derive(Serialize)]
struct TooltipBucket {
    label: String,
    x: String,
    countries: Vec<TooltipBreakdown>,
}

#[derive(Serialize)]
pub(crate) struct CountryTrafficChart {
    has_data: bool,
    countries: Vec<CountryArea>,
    x_labels: Vec<XAxisLabel>,
    y_labels: Vec<AxisLabel>,
    peak: i64,
    buckets_json: String,
}

struct CountrySeries {
    time_buckets: Vec<chrono::DateTime<chrono::Utc>>,
    country_names: Vec<String>,
    series: Vec<Vec<f64>>,
}

fn collect_country_series(data: &[crate::admin::types::TrafficCountryBucket]) -> CountrySeries {
    let time_buckets: Vec<chrono::DateTime<chrono::Utc>> = data
        .iter()
        .map(|r| r.bucket)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    let mut bucket_map: BTreeMap<chrono::DateTime<chrono::Utc>, BTreeMap<String, i64>> =
        BTreeMap::new();
    let mut country_totals: BTreeMap<String, i64> = BTreeMap::new();
    for r in data {
        bucket_map
            .entry(r.bucket)
            .or_default()
            .insert(r.country.clone(), r.sessions);
        *country_totals.entry(r.country.clone()).or_default() += r.sessions;
    }

    let mut countries_ordered: Vec<(String, i64)> = country_totals.into_iter().collect();
    countries_ordered.sort_by(|a, b| b.1.cmp(&a.1));
    let country_names: Vec<String> = countries_ordered
        .iter()
        .map(|(name, _)| name.clone())
        .collect();

    let series: Vec<Vec<f64>> = country_names
        .iter()
        .map(|name| {
            time_buckets
                .iter()
                .map(|tb| {
                    let val = bucket_map
                        .get(tb)
                        .and_then(|m| m.get(name))
                        .copied()
                        .unwrap_or(0);
                    f64::from(i32::try_from(val).unwrap_or(0))
                })
                .collect()
        })
        .collect();

    CountrySeries {
        time_buckets,
        country_names,
        series,
    }
}

fn build_cumulative_stacks(series: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let mut cumulative: Vec<Vec<f64>> = Vec::with_capacity(series.len());
    for (i, s) in series.iter().enumerate() {
        if i == 0 {
            cumulative.push(s.clone());
        } else {
            let prev = &cumulative[i - 1];
            cumulative.push(s.iter().zip(prev.iter()).map(|(val, p)| val + p).collect());
        }
    }
    cumulative
}

fn build_country_areas(
    country_names: &[String],
    cumulative: &[Vec<f64>],
    n: usize,
    svg_w: f64,
    svg_h: f64,
    y_max: f64,
) -> Vec<CountryArea> {
    country_names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let top = &cumulative[i];
            let base = if i == 0 {
                vec![0.0f64; n]
            } else {
                cumulative[i - 1].clone()
            };
            CountryArea {
                name: name.clone(),
                color: country_color(i),
                area_path: build_stacked_area(top, &base, n, svg_w, svg_h, y_max),
            }
        })
        .collect()
}

fn build_country_tooltips(
    time_buckets: &[chrono::DateTime<chrono::Utc>],
    country_names: &[String],
    country_series: &[Vec<f64>],
    n: usize,
    svg_w: f64,
    range: &str,
) -> Vec<TooltipBucket> {
    let tooltip_fmt = match range {
        "7d" | "30d" => "%b %d",
        _ => "%H:%M",
    };
    time_buckets
        .iter()
        .enumerate()
        .map(|(i, tb)| {
            let x = svg_x(i, n, svg_w);
            let countries = country_names
                .iter()
                .enumerate()
                .map(|(ci, name)| TooltipBreakdown {
                    country: name.clone(),
                    sessions: f64_to_i64(country_series[ci][i]),
                    color: country_color(ci),
                })
                .collect();
            TooltipBucket {
                label: tb.format(tooltip_fmt).to_string(),
                x: format!("{x:.1}"),
                countries,
            }
        })
        .collect()
}

pub(crate) fn compute_country_traffic_chart(
    data: &[crate::admin::types::TrafficCountryBucket],
    range: &str,
) -> CountryTrafficChart {
    if data.is_empty() {
        return CountryTrafficChart {
            has_data: false,
            countries: Vec::new(),
            x_labels: Vec::new(),
            y_labels: Vec::new(),
            peak: 0,
            buckets_json: "[]".to_string(),
        };
    }

    let svg_w: f64 = SVG_WIDTH;
    let svg_h: f64 = SVG_HEIGHT;

    let cs = collect_country_series(data);
    let n = cs.time_buckets.len();
    let cumulative = build_cumulative_stacks(&cs.series);

    let peak_f64 = cumulative
        .last()
        .map_or(1.0, |top| top.iter().copied().fold(0.0f64, f64::max))
        .max(1.0);
    let peak = f64_to_i64(peak_f64);
    let y_max = peak_f64 * 1.1;

    let countries = build_country_areas(&cs.country_names, &cumulative, n, svg_w, svg_h, y_max);

    let x_fmt = match range {
        "7d" => "%a %d",
        "30d" => "%b %d",
        _ => "%H:%M",
    };
    let step = (n / 7).max(1);
    let x_labels: Vec<XAxisLabel> = cs
        .time_buckets
        .iter()
        .enumerate()
        .filter_map(|(i, b)| {
            if i % step == 0 || i == n - 1 {
                let x = svg_x(i, n, svg_w);
                Some(XAxisLabel {
                    label: b.format(x_fmt).to_string(),
                    x: format!("{x:.1}"),
                })
            } else {
                None
            }
        })
        .collect();

    let y_labels = build_y_labels(peak, svg_h, y_max);
    let tooltips = build_country_tooltips(
        &cs.time_buckets,
        &cs.country_names,
        &cs.series,
        n,
        svg_w,
        range,
    );

    CountryTrafficChart {
        has_data: true,
        countries,
        x_labels,
        y_labels,
        peak,
        buckets_json: serde_json::to_string(&tooltips).unwrap_or_else(|_| String::new()),
    }
}
