use std::collections::BTreeMap;

use super::ssr_dashboard_traffic::compute_pct_change;
use super::ssr_dashboard_types::TopPageEnhancedView;
use super::ssr_dashboard_types::TopPageHorizon;
use crate::admin::numeric;

use super::ssr_dashboard_traffic::format_time_ms;

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

pub(super) fn build_top_pages_enhanced(
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

    pages.sort_unstable_by(|a, b| b.horizon_1d.views.cmp(&a.horizon_1d.views));
    pages
}
