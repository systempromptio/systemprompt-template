//! View-model assembly for the site analytics dashboard.
//!
//! Pure functions turning repository rows + the parsed query into the typed
//! context the `analytics-dashboard` template consumes. All percentage and
//! label math happens here so the template never derives a scale.

use crate::handlers::ssr::format::format_cost;
use crate::handlers::ssr::types::{
    LineChartSpec, MeterInput, MeterView, PieSliceInput, PieView, SvgLineChartView, SvgSeriesInput,
    delta_view, line_chart, meter_view, pie_view, sparkline,
};
use crate::repositories::analytics::site::distribution::ModelDistributionRow;
use crate::repositories::analytics::site::kpis::SiteKpis;
use crate::repositories::analytics::site::series::UsageBucket;
use crate::repositories::organizations::metrics::OrganizationMetrics;
use crate::util::time_range::TimeRange;

use super::context::{DashboardTimeRange, KpiStripView};
use super::{AnalyticsDashboardQuery, BASE_URL, urls};

pub(super) fn time_range_view(
    query: &AnalyticsDashboardQuery,
    range: &TimeRange,
) -> DashboardTimeRange {
    let preset = query.preset.clone().unwrap_or_else(|| {
        if query.from.is_some() && query.to.is_some() {
            "custom".to_owned()
        } else {
            "7d".to_owned()
        }
    });
    let qs = urls::preserved_query_string(query, &["preset", "from", "to", "page"]);
    let q_suffix = if qs.is_empty() {
        String::new()
    } else {
        format!("&{qs}")
    };
    DashboardTimeRange {
        preset,
        from: range.from.to_rfc3339(),
        to: range.to.to_rfc3339(),
        base_url: BASE_URL,
        query: q_suffix,
    }
}

pub(super) fn kpi_strip(
    kpis: &SiteKpis,
    wasted_seats: i64,
    range: &TimeRange,
    query: &AnalyticsDashboardQuery,
    series: Option<&[UsageBucket]>,
) -> KpiStripView {
    let days = window_days(range);
    let cost_per_request = if kpis.total_requests > 0 {
        kpis.total_cost_microdollars / kpis.total_requests
    } else {
        0
    };
    let per_user_day = if kpis.active_users > 0 {
        kpis.total_requests as f64 / kpis.active_users as f64 / days
    } else {
        0.0
    };
    let error_rate = if kpis.total_requests > 0 {
        kpis.error_count as f64 / kpis.total_requests as f64 * 100.0
    } else {
        0.0
    };
    let wasted_qs = urls::preserved_query_string(query, &["tab", "page"]);
    let wasted_url = if wasted_qs.is_empty() {
        format!("{BASE_URL}?tab=seats")
    } else {
        format!("{BASE_URL}?tab=seats&{wasted_qs}")
    };

    KpiStripView {
        requests: kpis.total_requests,
        error_display: format!("{} errors ({error_rate:.1}%)", kpis.error_count),
        cost_display: format_cost(kpis.total_cost_microdollars),
        cost_per_request_display: format_cost(cost_per_request),
        weekly_active_users: kpis.weekly_active_users,
        active_users: kpis.active_users,
        requests_per_user_day_display: format!("{per_user_day:.1}"),
        wasted_seats,
        wasted_seats_url: wasted_url,
        tokens_display: compact(kpis.total_tokens),
        // Why: polarity is stated per metric — more requests and more active
        // users read as good, more spend does not.
        requests_delta: delta_view(kpis.total_requests, kpis.prev_total_requests, true),
        cost_delta: delta_view(
            kpis.total_cost_microdollars,
            kpis.prev_total_cost_microdollars,
            false,
        ),
        wau_delta: delta_view(
            kpis.weekly_active_users,
            kpis.prev_weekly_active_users,
            true,
        ),
        tokens_delta: delta_view(kpis.total_tokens, kpis.prev_total_tokens, true),
        // Why: built from the series the page already loaded — a sparkline
        // never costs an extra query, so tabs without one simply omit it.
        requests_spark: series
            .map(|b| sparkline(&b.iter().map(|x| x.requests).collect::<Vec<_>>())),
        cost_spark: series
            .map(|b| sparkline(&b.iter().map(|x| x.cost_microdollars).collect::<Vec<_>>())),
    }
}

pub(super) fn window_days(range: &TimeRange) -> f64 {
    let secs = (range.to - range.from).num_seconds().max(1) as f64;
    (secs / 86_400.0).max(1.0)
}

pub(super) fn volume_chart(
    buckets: &[UsageBucket],
    range: &TimeRange,
    weekly: bool,
) -> SvgLineChartView {
    let total: i64 = buckets.iter().map(|b| b.requests).sum();
    let errors: i64 = buckets.iter().map(|b| b.errors).sum();
    let label = if weekly {
        "requests/week"
    } else {
        "requests/day"
    };
    line_chart(LineChartSpec {
        title: "Request volume",
        subtitle: format!("{total} requests · {errors} failed"),
        empty_message: "No gateway requests in this window.",
        series: vec![SvgSeriesInput {
            label: label.to_owned(),
            values: buckets.iter().map(|b| b.requests).collect(),
            value_display: total.to_string(),
        }],
        ref_lines: Vec::new(),
        y_max: None,
        y_display: |v| v.to_string(),
        x_start_display: date_label(range.from),
        x_mid_display: date_label(midpoint(range)),
        x_end_display: date_label(range.to),
        show_area: true,
    })
}

pub(super) fn spend_chart(
    buckets: &[UsageBucket],
    range: &TimeRange,
    weekly: bool,
) -> SvgLineChartView {
    let total: i64 = buckets.iter().map(|b| b.cost_microdollars).sum();
    let label = if weekly { "cost/week" } else { "cost/day" };
    line_chart(LineChartSpec {
        title: "Cost over time",
        subtitle: format!("{} across the window", format_cost(total)),
        empty_message: "No billed requests in this window.",
        series: vec![SvgSeriesInput {
            label: label.to_owned(),
            values: buckets.iter().map(|b| b.cost_microdollars).collect(),
            value_display: format_cost(total),
        }],
        ref_lines: Vec::new(),
        y_max: None,
        y_display: format_cost,
        x_start_display: date_label(range.from),
        x_mid_display: date_label(midpoint(range)),
        x_end_display: date_label(range.to),
        show_area: true,
    })
}

pub(super) fn model_pie(
    models: &[ModelDistributionRow],
    query: &AnalyticsDashboardQuery,
) -> PieView {
    let total_requests: i64 = models.iter().map(|m| m.requests).sum();
    let slices = models
        .iter()
        .map(|m| PieSliceInput {
            label: m.model.clone(),
            value: m.requests,
            value_display: format!(
                "{} req · {} tok · {}",
                m.requests,
                compact(m.tokens),
                format_cost(m.cost_microdollars)
            ),
            filter_url: Some(model_log_url(query, &m.model)),
        })
        .collect();
    pie_view(
        "Model usage",
        format!("{total_requests} requests across {} models", models.len()),
        slices,
        "No model usage in this window.",
    )
}

fn model_log_url(query: &AnalyticsDashboardQuery, model: &str) -> String {
    let mut url = format!(
        "/admin/entities/requests?tab=log&model={}",
        urlencoding::encode(model)
    );
    if let Some(user) = query.user_id.as_ref().filter(|u| !u.as_str().is_empty()) {
        url.push_str(&format!("&user_id={}", urlencoding::encode(user.as_str())));
    }
    url
}

pub(super) fn org_meters(orgs: &[OrganizationMetrics]) -> Vec<MeterView> {
    orgs.iter()
        .map(|o| {
            meter_view(MeterInput {
                label: o.name.clone(),
                spent_microdollars: o.cost_microdollars_mtd,
                cap_microdollars: o.cap_microdollars,
                warn_microdollars: o.warn_microdollars,
            })
        })
        .collect()
}


pub(super) fn compact(v: i64) -> String {
    if v >= 1_000_000 {
        format!("{:.1}M", v as f64 / 1_000_000.0)
    } else if v >= 10_000 {
        format!("{}k", v / 1000)
    } else if v >= 1000 {
        format!("{:.1}k", v as f64 / 1000.0)
    } else {
        v.to_string()
    }
}

pub(super) fn midpoint(range: &TimeRange) -> chrono::DateTime<chrono::Utc> {
    range.from + (range.to - range.from) / 2
}

pub(super) fn date_label(ts: chrono::DateTime<chrono::Utc>) -> String {
    ts.with_timezone(&chrono::Local).format("%b %d").to_string()
}

pub(super) fn format_date(ts: chrono::DateTime<chrono::Utc>) -> String {
    ts.with_timezone(&chrono::Local)
        .format("%Y-%m-%d")
        .to_string()
}

pub(super) fn bucket_label(ts: chrono::DateTime<chrono::Utc>, weekly: bool) -> String {
    let local = ts.with_timezone(&chrono::Local);
    if weekly {
        format!("week of {}", local.format("%b %d"))
    } else {
        local.format("%b %d").to_string()
    }
}
