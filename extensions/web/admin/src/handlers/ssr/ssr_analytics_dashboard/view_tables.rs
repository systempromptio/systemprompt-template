//! Table builders for the usage and seats tabs: the leaderboard, the
//! permission-grant stat, seat summaries, and the wasted-seats rows.
//! Split from `view.rs` at the 300-line ceiling.

use crate::handlers::ssr::format::format_cost;
use crate::handlers::ssr::types::bar_pct;
use crate::repositories::analytics::site::kpis::PermissionGrantStats;
use crate::repositories::analytics::site::leaderboards::UserUsageRow;
use crate::repositories::analytics::site::seats::InactiveSeatRow;
use crate::repositories::organizations::metrics::OrganizationMetrics;
use crate::util::time_range::TimeRange;

use super::context::{LeaderRowView, PermissionStatsView, SeatSummaryView, WastedSeatView};
use super::view::{compact, format_date, window_days};
use super::{AnalyticsDashboardQuery, urls};

pub(super) fn leaderboard_rows(
    rows: &[UserUsageRow],
    range: &TimeRange,
    query: &AnalyticsDashboardQuery,
) -> Vec<LeaderRowView> {
    let max = rows.iter().map(|r| r.requests).max().unwrap_or(0);
    let days = window_days(range);
    rows.iter()
        .map(|r| LeaderRowView {
            user_id: r.user_id.clone(),
            label: r.label.clone(),
            department: r.department.clone(),
            requests: r.requests,
            share_pct: bar_pct(r.requests, max),
            tokens_display: compact(r.tokens),
            cost_display: format_cost(r.cost_microdollars),
            requests_per_day_display: format!("{:.1}", r.requests as f64 / days),
            last_active_display: r.last_active.map_or_else(|| "—".to_owned(), format_date),
            scope_url: urls::scope_to_user_url(query, &r.user_id),
            log_url: format!(
                "/admin/entities/requests?tab=log&user_id={}",
                urlencoding::encode(r.user_id.as_str())
            ),
            detail_url: format!(
                "/admin/access/user?id={}",
                urlencoding::encode(r.user_id.as_str())
            ),
        })
        .collect()
}

pub(super) fn permission_stats(stats: &PermissionGrantStats) -> PermissionStatsView {
    let rate = if stats.requests > 0 {
        stats.granted as f64 / stats.requests as f64 * 100.0
    } else {
        0.0
    };
    PermissionStatsView {
        requests: stats.requests,
        granted: stats.granted,
        rate_display: format!("{rate:.0}%"),
        has_data: stats.requests > 0,
    }
}

pub(super) fn seat_summaries(orgs: &[OrganizationMetrics]) -> Vec<SeatSummaryView> {
    orgs.iter()
        .map(|o| {
            let (limit_display, pct) = match o.seat_limit {
                Some(limit) if limit > 0 => (
                    limit.to_string(),
                    (o.seats_used.saturating_mul(100) / i64::from(limit)).clamp(0, 100),
                ),
                _ => ("unlimited".to_owned(), 0),
            };
            SeatSummaryView {
                org_name: o.name.clone(),
                seats_used: o.seats_used,
                seat_limit_display: limit_display,
                pct,
            }
        })
        .collect()
}

pub(super) fn wasted_seat_rows(rows: &[InactiveSeatRow]) -> Vec<WastedSeatView> {
    rows.iter()
        .map(|r| WastedSeatView {
            user_id: r.user_id.clone(),
            label: r.label.clone(),
            email: r.email.clone(),
            department: r.department.clone(),
            org_name: r.org_name.clone(),
            last_request_display: r
                .last_request_at
                .map_or_else(|| "never".to_owned(), format_date),
            detail_url: format!(
                "/admin/access/user?id={}",
                urlencoding::encode(r.user_id.as_str())
            ),
        })
        .collect()
}
