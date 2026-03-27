use serde_json::json;

use std::collections::BTreeMap;

use super::ssr_dashboard_types::{
    ContentPerformanceView, DeviceBar, GeoBar, RealtimePulseView, SourceBar, TopPageEnhancedView,
    TopPageHorizon, TopPageView, TrafficKpisView, TrafficResult,
};
use crate::admin::numeric;

type TrafficBreakdowns = (
    serde_json::Value,
    Vec<SourceBar>,
    Vec<GeoBar>,
    Vec<DeviceBar>,
    Vec<TopPageView>,
    serde_json::Value,
    Vec<TopPageEnhancedView>,
);

pub(super) fn format_time_ms(ms: f64) -> String {
    let secs = numeric::round_to_i64(ms / 1000.0);
    if secs >= 60 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{secs}s")
    }
}

fn format_time_seconds(secs: f64) -> String {
    let s = numeric::round_to_i64(secs);
    if s >= 60 {
        format!("{}m {}s", s / 60, s % 60)
    } else {
        format!("{s}s")
    }
}

pub(super) fn compute_pct_change(current: f64, previous: f64) -> (String, String, String) {
    if previous.abs() < f64::EPSILON {
        if current > 0.0 {
            return ("new".to_string(), "up".to_string(), "trend-up".to_string());
        }
        return (
            "-".to_string(),
            "flat".to_string(),
            "trend-flat".to_string(),
        );
    }
    let change = ((current - previous) / previous * 100.0).round();
    if change > 0.0 {
        (
            format!("+{change:.0}%"),
            "up".to_string(),
            "trend-up".to_string(),
        )
    } else if change < 0.0 {
        (
            format!("{change:.0}%"),
            "down".to_string(),
            "trend-down".to_string(),
        )
    } else {
        (
            "0%".to_string(),
            "flat".to_string(),
            "trend-flat".to_string(),
        )
    }
}

fn build_shared_data(
    realtime_pulse: Option<&crate::admin::types::RealtimePulse>,
    content_performance: &[crate::admin::types::ContentPerformanceRow],
) -> (Option<RealtimePulseView>, Vec<ContentPerformanceView>) {
    let pulse = realtime_pulse.map(|rp| RealtimePulseView {
        sessions_this_hour: rp.sessions_this_hour,
        page_views_this_hour: rp.page_views_this_hour,
        unique_visitors_today: rp.unique_visitors_today,
    });

    let perf: Vec<ContentPerformanceView> = content_performance
        .iter()
        .map(|c| ContentPerformanceView {
            title: c.title.clone(),
            views: c.views,
            trend: c.trend.clone(),
            avg_time: format_time_seconds(c.avg_time_seconds),
        })
        .collect();

    (pulse, perf)
}

fn build_traffic_kpis(k: &crate::admin::types::TrafficKpis) -> TrafficKpisView {
    let (sessions_change, sessions_dir, sessions_class) = compute_pct_change(
        numeric::to_f64(k.sessions_current),
        numeric::to_f64(k.sessions_previous),
    );
    let (pv_change, pv_dir, pv_class) = compute_pct_change(
        numeric::to_f64(k.page_views_current),
        numeric::to_f64(k.page_views_previous),
    );
    let (time_change, time_dir, time_class) =
        compute_pct_change(k.avg_time_ms_current, k.avg_time_ms_previous);
    let (uv_change, uv_dir, uv_class) = compute_pct_change(
        numeric::to_f64(k.unique_visitors_current),
        numeric::to_f64(k.unique_visitors_previous),
    );

    TrafficKpisView {
        sessions: k.sessions_current,
        sessions_change,
        sessions_dir,
        sessions_class,
        page_views: k.page_views_current,
        pv_change,
        pv_dir,
        pv_class,
        avg_time: format_time_ms(k.avg_time_ms_current),
        time_change,
        time_dir,
        time_class,
        unique_visitors: k.unique_visitors_current,
        uv_change,
        uv_dir,
        uv_class,
    }
}

fn build_horizon(
    buckets: &[(chrono::NaiveDate, i64, i64, f64)],
    days: i64,
    today: chrono::NaiveDate,
) -> TopPageHorizon {
    let cutoff = today - chrono::Duration::days(days);
    let in_range: Vec<_> = buckets.iter().filter(|(d, _, _, _)| *d > cutoff).collect();

    let views: i64 = in_range.iter().map(|(_, v, _, _)| v).sum();
    let sessions: i64 = in_range.iter().map(|(_, _, s, _)| s).sum();
    let total_events: i64 = in_range.iter().map(|(_, v, _, _)| v).sum();
    let avg_time_ms = if total_events > 0 {
        in_range
            .iter()
            .map(|(_, v, _, t)| numeric::to_f64(*v) * t)
            .sum::<f64>()
            / numeric::to_f64(total_events)
    } else {
        0.0
    };

    let sparkline_values: Vec<i64> = in_range.iter().map(|(_, v, _, _)| *v).collect();
    let views_sparkline = format!(
        "[{}]",
        sparkline_values
            .iter()
            .map(i64::to_string)
            .collect::<Vec<_>>()
            .join(",")
    );

    let prev_cutoff = cutoff - chrono::Duration::days(days);
    let prev_views: i64 = buckets
        .iter()
        .filter(|(d, _, _, _)| *d > prev_cutoff && *d <= cutoff)
        .map(|(_, v, _, _)| v)
        .sum();

    let (views_change, _, views_trend) =
        compute_pct_change(numeric::to_f64(views), numeric::to_f64(prev_views));

    TopPageHorizon {
        views,
        sessions,
        avg_time: format_time_ms(avg_time_ms),
        views_sparkline,
        views_trend,
        views_change,
    }
}

fn build_top_pages_enhanced(
    daily_buckets: &[crate::admin::types::TopPageDailyBucket],
) -> Vec<TopPageEnhancedView> {
    let today = chrono::Utc::now().date_naive();

    let mut by_page: BTreeMap<String, Vec<(chrono::NaiveDate, i64, i64, f64)>> = BTreeMap::new();
    for b in daily_buckets {
        by_page.entry(b.page_url.clone()).or_default().push((
            b.day,
            b.views,
            b.sessions,
            b.avg_time_ms,
        ));
    }

    let mut pages: Vec<TopPageEnhancedView> = by_page
        .into_iter()
        .map(|(page_url, buckets)| {
            let horizon_1d = build_horizon(&buckets, 1, today);
            let yesterday = today - chrono::Duration::days(1);
            let horizon_yesterday = build_horizon(&buckets, 1, yesterday);
            let horizon_week = build_horizon(&buckets, 7, today);
            let horizon_month = build_horizon(&buckets, 31, today);

            let page_label = if page_url.len() > 50 {
                format!("{}...", &page_url[..47])
            } else {
                page_url.clone()
            };

            TopPageEnhancedView {
                page_url,
                page_label,
                horizon_1d,
                horizon_yesterday,
                horizon_7d: horizon_week,
                horizon_31d: horizon_month,
            }
        })
        .collect();

    pages.sort_by(|a, b| b.horizon_1d.views.cmp(&a.horizon_1d.views));
    pages
}

fn build_traffic_breakdowns(
    t: &crate::admin::types::TrafficData,
    traffic_range_key: &str,
) -> TrafficBreakdowns {
    let chart = serde_json::to_value(super::charts::compute_traffic_chart_data(
        &t.timeseries,
        traffic_range_key,
    ))
    .unwrap_or_else(|_| serde_json::Value::Null);
    let country_chart = serde_json::to_value(super::charts::compute_country_traffic_chart(
        &t.country_timeseries,
        traffic_range_key,
    ))
    .unwrap_or_else(|_| serde_json::Value::Null);

    let max_source = t.sources.first().map_or(1, |s| s.sessions).max(1);
    let sources: Vec<SourceBar> = t
        .sources
        .iter()
        .map(|s| SourceBar {
            source: s.source.clone(),
            sessions: s.sessions,
            pct: s.sessions.saturating_mul(100) / max_source,
        })
        .collect();

    let max_geo = t.geo.first().map_or(1, |g| g.sessions).max(1);
    let geo: Vec<GeoBar> = t
        .geo
        .iter()
        .map(|g| GeoBar {
            country: g.country.clone(),
            sessions: g.sessions,
            pct: g.sessions.saturating_mul(100) / max_geo,
        })
        .collect();

    let max_device = t.devices.first().map_or(1, |d| d.sessions).max(1);
    let devices: Vec<DeviceBar> = t
        .devices
        .iter()
        .map(|d| DeviceBar {
            device: d.device.clone(),
            sessions: d.sessions,
            pct: d.sessions.saturating_mul(100) / max_device,
        })
        .collect();

    let top_pages: Vec<TopPageView> = t
        .top_pages
        .iter()
        .map(|p| TopPageView {
            page_url: p.page_url.clone(),
            events: p.events,
            sessions: p.sessions,
            avg_time: format_time_ms(p.avg_time_ms),
        })
        .collect();

    let top_pages_enhanced = build_top_pages_enhanced(&t.top_pages_daily);

    (
        chart,
        sources,
        geo,
        devices,
        top_pages,
        country_chart,
        top_pages_enhanced,
    )
}

pub(super) fn build_traffic_data(
    traffic: Option<&crate::admin::types::TrafficData>,
    traffic_range_key: &str,
    realtime_pulse: Option<&crate::admin::types::RealtimePulse>,
    content_performance: &[crate::admin::types::ContentPerformanceRow],
) -> TrafficResult {
    let (realtime_pulse_view, content_performance_view) =
        build_shared_data(realtime_pulse, content_performance);

    if let Some(t) = traffic {
        let kpis = build_traffic_kpis(&t.kpis);
        let (chart, sources, geo, devices, top_pages, country_chart, top_pages_enhanced) =
            build_traffic_breakdowns(t, traffic_range_key);
        TrafficResult {
            has_traffic: true,
            kpis: Some(kpis),
            chart,
            sources,
            geo,
            devices,
            top_pages,
            top_pages_enhanced,
            country_chart,
            realtime_pulse: realtime_pulse_view,
            content_performance: content_performance_view,
        }
    } else {
        TrafficResult {
            has_traffic: false,
            kpis: None,
            chart: json!({"has_data": false}),
            sources: vec![],
            geo: vec![],
            devices: vec![],
            top_pages: vec![],
            top_pages_enhanced: vec![],
            country_chart: json!({"has_data": false}),
            realtime_pulse: realtime_pulse_view,
            content_performance: content_performance_view,
        }
    }
}
