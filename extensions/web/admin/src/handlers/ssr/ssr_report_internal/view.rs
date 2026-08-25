//! Row-to-view-model formatting for the internal P&L report.
//!
//! Pure functions over what the repository returned. Keeping them here means
//! the handler reads as "fetch, assemble, render" and every display rule —
//! how a share bar scales, when a budget pill turns red — has one home.

use crate::handlers::ssr::format::{format_cost, format_token_total};
use crate::handlers::ssr::types::{BudgetState, ChartBarView, ChartView, bar_pct};
use crate::repositories::reports::internal::{
    OrganizationMonthPnl, PlatformMonthPoint, SupplierMonthCost,
};

use super::context::{InternalTotals, OrgPnlView, SupplierView};

pub(super) const fn margin_variant(margin_microdollars: i64) -> &'static str {
    if margin_microdollars < 0 {
        "stat-card--negative"
    } else {
        ""
    }
}

pub(super) fn org_view(row: &OrganizationMonthPnl, month_key: &str) -> OrgPnlView {
    let margin = row.margin_microdollars();
    let budget_pct = row.budget_used_pct();
    OrgPnlView {
        detail_url: format!("/admin/enterprises/{}", row.slug),
        customer_report_url: format!("/admin/reports/customer?org={}&month={month_key}", row.slug),
        slug: row.slug.clone(),
        name: row.name.clone(),
        plan_name: row.plan_name.clone(),
        is_active: row.status == "active",
        status: row.status.clone(),
        is_platform: row.is_platform,
        seats_used: row.seats_used,
        seat_limit: row.seat_limit,
        has_seat_limit: row.seat_limit.is_some(),
        active_users: row.active_users,
        requests: row.requests,
        tokens: row.tokens,
        tokens_display: format_token_total(row.tokens),
        revenue_microdollars: row.revenue_microdollars,
        cost_microdollars: row.cost_microdollars,
        margin_microdollars: margin,
        margin_positive: margin >= 0,
        margin_pct_display: pct_display(row.margin_pct()),
        cost_per_seat_microdollars: row.cost_per_seat_microdollars(),
        budget_pct,
        has_budget: budget_pct.is_some(),
        budget_state: budget_pct.map_or("ok", |p| BudgetState::from_pct(p).as_str()),
    }
}

// Why: the platform tenant is included in cost — its spend is real money —
// and excluded from the customer count and from revenue, because it bills
// nobody and would otherwise drag the portfolio margin down as if a customer
// were underwater.
pub(super) fn totals(rows: &[OrganizationMonthPnl]) -> InternalTotals {
    let revenue: i64 = rows
        .iter()
        .filter(|r| !r.is_platform)
        .map(|r| r.revenue_microdollars)
        .sum();
    let cost: i64 = rows.iter().map(|r| r.cost_microdollars).sum();
    let tokens: i64 = rows.iter().map(|r| r.tokens).sum();
    let requests: i64 = rows.iter().map(|r| r.requests).sum();
    let margin = revenue - cost;
    let margin_pct = (revenue > 0).then(|| margin.saturating_mul(100) / revenue);

    InternalTotals {
        customers: rows.iter().filter(|r| !r.is_platform).count() as i64,
        seats: rows.iter().map(|r| r.seats_used).sum(),
        active_users: rows.iter().map(|r| r.active_users).sum(),
        requests,
        tokens,
        tokens_display: format_token_total(tokens),
        revenue_microdollars: revenue,
        cost_microdollars: cost,
        margin_microdollars: margin,
        margin_positive: margin >= 0,
        margin_variant: margin_variant(margin),
        margin_pct,
        margin_pct_display: pct_display(margin_pct),
        cost_per_1m_display: per_unit_display(cost, tokens, 1_000_000),
        cost_per_request_display: per_unit_display(cost, requests, 1),
    }
}

pub(super) fn supplier_views(rows: &[SupplierMonthCost]) -> Vec<SupplierView> {
    let max = rows.iter().map(|r| r.cost_microdollars).max().unwrap_or(0);
    rows.iter()
        .map(|r| SupplierView {
            key: r.key.clone(),
            requests: r.requests,
            tokens_display: format_token_total(r.tokens),
            cost_microdollars: r.cost_microdollars,
            share_pct: bar_pct(r.cost_microdollars, max),
            share_display: format_cost(r.cost_microdollars),
        })
        .collect()
}

pub(super) fn trend_chart(points: &[PlatformMonthPoint]) -> ChartView {
    let max = points
        .iter()
        .map(|p| p.cost_microdollars)
        .max()
        .unwrap_or(0);
    let total: i64 = points.iter().map(|p| p.cost_microdollars).sum();
    let (first, last) = (points.first(), points.last());
    ChartView {
        title: "Provider cost by month",
        subtitle: format!("{} across the window", format_cost(total)),
        tone: "success",
        series: points
            .iter()
            .map(|p| ChartBarView {
                pct: bar_pct(p.cost_microdollars, max),
                tooltip: format!(
                    "{}: {} over {} requests",
                    month_label(p),
                    format_cost(p.cost_microdollars),
                    p.requests
                ),
            })
            .collect(),
        has_data: max > 0,
        y_max_display: format_cost(max),
        y_mid_display: format_cost(max / 2),
        x_start_display: first.map(month_label).unwrap_or_default(),
        x_mid_display: points
            .get(points.len() / 2)
            .map(month_label)
            .unwrap_or_default(),
        x_end_display: last.map(month_label).unwrap_or_default(),
        empty_message: "No billed requests in these months.",
    }
}

fn month_label(point: &PlatformMonthPoint) -> String {
    point.month_start.format("%b %Y").to_string()
}

fn pct_display(pct: Option<i64>) -> String {
    pct.map_or_else(|| "—".to_owned(), |p| format!("{p}%"))
}

fn per_unit_display(cost: i64, units: i64, per: i64) -> String {
    if units <= 0 {
        return "—".to_owned();
    }
    format_cost(cost.saturating_mul(per) / units)
}
