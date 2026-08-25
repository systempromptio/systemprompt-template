//! Spend-tab view builders: the month-to-date burn-up against the plan's
//! caps, the soft-cap crossing history, the fast/slow latency split, and the
//! client-reported session-cost cards.
//!
//! Split from `view.rs` at the 300-line ceiling; the shared label helpers
//! stay there and are imported here.

use crate::handlers::ssr::format::{format_cost, format_duration_ms};
use crate::handlers::ssr::types::{LineChartSpec, SvgLineChartView, SvgSeriesInput, line_chart};
use crate::repositories::analytics::site::anomalies::UsageAnomalyRow;
use crate::repositories::analytics::site::latency::LatencySplit;
use crate::repositories::analytics::site::series::UsageBucket;
use crate::repositories::analytics::site::session_costs::SessionCostStats;
use crate::repositories::organizations::budget_warnings::BudgetWarningHistoryRow;
use crate::repositories::organizations::metrics::OrganizationMetrics;
use crate::util::svg;

use super::context::{AnomalyRowView, BudgetWarningRowView, FastSlowView, SessionCostsView};
use super::view::compact;

// Why: the y-axis is pinned to the cap (when there is one) so the cap line is
// always on-plot — a burn-up that scaled to the data alone would hide the very
// line it exists to compare against. The projection is plainly linear and the
// subtitle says so.
pub(super) fn burndown_chart(mtd: &[UsageBucket], org: &OrganizationMetrics) -> SvgLineChartView {
    let daily: Vec<i64> = mtd.iter().map(|b| b.cost_microdollars).collect();
    let cumulative = svg::cumulative(&daily);
    let spent = cumulative.last().copied().unwrap_or(0);

    let mut ref_lines = Vec::new();
    if let Some(warn) = org.warn_microdollars {
        ref_lines.push((warn, format!("soft cap {}", format_cost(warn)), "warn"));
    }
    if let Some(cap) = org.cap_microdollars {
        ref_lines.push((cap, format!("hard cap {}", format_cost(cap)), "over"));
    }

    let day_of_month = mtd.len().max(1) as i64;
    let days_in_month = 30i64;
    let pace = spent / day_of_month * days_in_month;
    let subtitle = org.cap_microdollars.map_or_else(
        || {
            format!(
                "{} month-to-date · on pace for ~{} (linear) · uncapped plan",
                format_cost(spent),
                format_cost(pace)
            )
        },
        |cap| {
            format!(
                "{} of {} month-to-date · on pace for ~{} (linear)",
                format_cost(spent),
                format_cost(cap),
                format_cost(pace)
            )
        },
    );

    let y_max = org
        .cap_microdollars
        .map(|cap| svg::nice_max(cap.max(spent)));

    line_chart(LineChartSpec {
        title: "Month-to-date spend",
        subtitle,
        empty_message: "No billed requests this month.",
        series: vec![SvgSeriesInput {
            label: "cumulative spend".to_owned(),
            values: cumulative,
            value_display: format_cost(spent),
        }],
        ref_lines,
        y_max,
        y_display: format_cost,
        x_start_display: "month start".to_owned(),
        x_mid_display: String::new(),
        x_end_display: "today".to_owned(),
        show_area: true,
    })
}

// Why: cost renders as dollars and the counting metrics as counts; the rows
// come from one table, so the discrimination lives here rather than in SQL.
pub(super) fn anomaly_rows(rows: &[UsageAnomalyRow]) -> Vec<AnomalyRowView> {
    rows.iter()
        .map(|r| {
            let (observed, baseline) = if r.metric == "cost" {
                (format_cost(r.observed), format_cost(r.baseline))
            } else {
                (r.observed.to_string(), r.baseline.to_string())
            };
            AnomalyRowView {
                metric: r.metric.clone(),
                window_display: r.window_start.format("%Y-%m-%d %H:%M UTC").to_string(),
                observed_display: observed,
                baseline_display: baseline,
            }
        })
        .collect()
}

pub(super) fn budget_warning_rows(rows: &[BudgetWarningHistoryRow]) -> Vec<BudgetWarningRowView> {
    rows.iter()
        .map(|r| BudgetWarningRowView {
            org_name: r.org_name.clone(),
            kind_display: if r.kind == "forecast_overrun" {
                "Forecast overrun"
            } else {
                "Soft cap"
            },
            month_display: r.month.format("%B %Y").to_string(),
            threshold_display: format_cost(r.threshold_microdollars),
            spent_display: format_cost(r.spent_microdollars),
            over_by_display: format_cost((r.spent_microdollars - r.threshold_microdollars).max(0)),
            first_seen_display: r.first_seen_at.format("%Y-%m-%d").to_string(),
            last_seen_display: r.last_seen_at.format("%Y-%m-%d").to_string(),
        })
        .collect()
}

// Why: this platform has no fast/slow request pools, so the split is stated
// as what it actually is — a latency bucket at the caller's SLO threshold —
// with the percentiles and breach share beside it and untimed requests shown
// rather than folded away. The displays derive from the threshold the query
// actually bound, so the caption can never contradict the split.
pub(super) fn fast_slow(split: &LatencySplit) -> FastSlowView {
    let timed = split.fast + split.slow;
    FastSlowView {
        fast: split.fast,
        slow: split.slow,
        untimed: split.untimed,
        threshold_display: format_duration_ms(i64::from(split.threshold_ms)),
        breach_pct_display: if timed > 0 {
            let permille = split.slow.saturating_mul(1000) / timed;
            format!("{}.{}%", permille / 10, permille % 10)
        } else {
            "–".to_owned()
        },
        p50_display: format_duration_ms(split.p50_ms.round() as i64),
        p95_display: format_duration_ms(split.p95_ms.round() as i64),
        has_data: timed + split.untimed > 0,
    }
}

pub(super) fn session_costs(stats: &SessionCostStats) -> SessionCostsView {
    SessionCostsView {
        has_data: stats.sessions > 0,
        sessions: stats.sessions,
        cache_hit_display: format!("{:.0}%", stats.cache_hit_pct),
        cache_read_display: compact(stats.cache_read_tokens),
        avg_context_display: compact(stats.avg_context_window),
        max_context_display: compact(stats.max_context_window),
    }
}

// Why: a burn-up against a cap is only meaningful for a single organization;
// with none (or all) in scope the page says so instead of summing unrelated
// contracts into one line.
pub(super) fn burndown_view(
    tab: super::context::DashboardTab,
    fetched: &super::data::AnalyticsDashboardData,
) -> Option<SvgLineChartView> {
    if tab != super::context::DashboardTab::Spend || fetched.org_metrics.len() != 1 {
        return None;
    }
    fetched
        .org_metrics
        .first()
        .map(|org| burndown_chart(&fetched.mtd_series, org))
}
