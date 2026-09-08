//! Table builders for the usage tab: the leaderboard and the
//! permission-grant stat.
//! Split from `view.rs` at the 300-line ceiling.

use crate::handlers::ssr::format::format_cost;
use crate::handlers::ssr::types::bar_pct;
use crate::repositories::analytics::site::kpis::PermissionGrantStats;
use crate::repositories::analytics::site::leaderboards::{LeaderboardSort, UserUsageRow};
use crate::util::time_range::TimeRange;

use crate::handlers::ssr::list_view::PageWindow;

use super::context::{LeaderRowView, LeaderboardView, PermissionStatsView};
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
            groups_display: if r.groups.is_empty() {
                "Unassigned".to_owned()
            } else {
                r.groups.join(", ")
            },
            requests: r.requests,
            share_pct: bar_pct(r.requests, max),
            tokens_display: compact(r.tokens),
            cost_display: format_cost(r.cost_microdollars),
            requests_per_day_display: format!("{:.1}", r.requests as f64 / days),
            last_active_display: r.last_active.map_or_else(|| "—".to_owned(), format_date),
            scope_url: urls::scope_to_user_url(query, &r.user_id),
            log_url: format!(
                "/admin/requests?tab=log&user_id={}",
                urlencoding::encode(r.user_id.as_str())
            ),
            detail_url: format!("/admin/user?id={}", urlencoding::encode(r.user_id.as_str())),
            analytics_url: format!(
                "/admin/analytics/users/{}",
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

// Why: the same four keys the sort links offer, resolved through the same
// parser, so the header and the link bar can never disagree about the order.
fn sorted_key(query: &AnalyticsDashboardQuery) -> &'static str {
    let active = LeaderboardSort::from_sort_param(query.sort.as_deref());
    ["requests", "cost", "tokens", "last_active"]
        .into_iter()
        .find(|key| LeaderboardSort::from_sort_param(Some(key)) == active)
        .unwrap_or("requests")
}

pub(super) fn leaderboard_view(
    fetched: &super::data::AnalyticsDashboardData,
    range: &TimeRange,
    query: &AnalyticsDashboardQuery,
    page: i64,
) -> LeaderboardView {
    let rows = leaderboard_rows(&fetched.leaderboard, range, query);
    let pagination = urls::build_pagination(
        query,
        PageWindow::new(
            page,
            super::PAGE_SIZE,
            fetched.leaderboard_total,
            i64::try_from(fetched.leaderboard.len()).unwrap_or(super::PAGE_SIZE),
            "users",
        ),
    );
    LeaderboardView {
        sorted_key: sorted_key(query),
        has_rows: !rows.is_empty(),
        rows,
        sort_links: super::page::sort_links(query),
        pagination,
    }
}
