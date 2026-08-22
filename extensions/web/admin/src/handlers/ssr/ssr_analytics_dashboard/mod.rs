//! `/admin/analytics` — the site analytics dashboard.
//!
//! Five URL-driven tabs over one window: Overview (KPIs, request/cost trends,
//! model pie, spend meter), Usage (top-users leaderboard, permission grant
//! rate), Seats (utilisation and wasted seats), Spend (per-org soft/hard cap
//! meters), and Code (commit activity and AI line deltas). Drill-down state —
//! organization, department, user, window, bucket — all lives in the query
//! string.
//!
//! Scoping: admin-only. `?org=` is honoured only for platform admins; anyone
//! else is locked to their own organization and has no way to ask for
//! another's data (the `resolve_slug` rule from the customer report).

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::repositories::analytics::site::leaderboards::LeaderboardSort;
use crate::repositories::analytics::site::series::SeriesBucket;
use crate::repositories::analytics::site::{SiteScope, seats};
use crate::repositories::organizations::crud;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use crate::util::org_scope::OrgScope;
use crate::util::time_range::{
    TimeRange, TimeRangePreset, TimeRangeQuery, parse_time_range, preset_to_range,
};

mod context;
mod context_seats;
mod data;
mod filters;
mod urls;
mod view;
mod view_code;
mod view_models;
mod view_spend;
mod view_tables;

use context::{AnalyticsDashboardContext, DashboardTab, FiltersView};

const BASE_URL: &str = "/admin/analytics";
const PAGE_SIZE: i64 = 50;

#[derive(Debug, Deserialize)]
pub(crate) struct AnalyticsDashboardQuery {
    pub tab: Option<String>,
    pub preset: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub bucket: Option<String>,
    pub org: Option<String>,
    pub department: Option<String>,
    pub user_id: Option<systemprompt::identifiers::UserId>,
    pub sort: Option<String>,
    pub page: Option<i64>,
    pub inactive_days: Option<i32>,
}

pub(crate) async fn analytics_dashboard_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<AnalyticsDashboardQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let tab = DashboardTab::from_query(query.tab.as_deref());
    let bucket = SeriesBucket::from_bucket_param(query.bucket.as_deref());
    let sort = LeaderboardSort::from_sort_param(query.sort.as_deref());
    let range = resolve_range(&query);
    let page = query.page.unwrap_or(0).max(0);

    let own_org_slug = crud::find_organization_for_user(&pool, &user_ctx.user_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "find_organization_for_user failed");
            None
        });
    let scope = resolve_scope(&user_ctx, &query, own_org_slug.as_deref());
    let inactive_days = resolve_inactive_days(query.inactive_days);

    let fetched = data::load_dashboard_data(
        &pool,
        data::DashboardQueryPlan {
            tab,
            scope: &scope,
            range,
            bucket,
            sort,
            page_size: PAGE_SIZE,
            offset: page * PAGE_SIZE,
            all_orgs: user_ctx.is_platform_admin,
            own_org_slug: own_org_slug.as_deref(),
            inactive_days,
        },
    )
    .await;

    let filters = filters::build_filters(&pool, &user_ctx, &query, &scope, bucket).await;
    let ctx = page_context(PageInput {
        query: &query,
        tab,
        range,
        bucket,
        page,
        is_platform_admin: user_ctx.is_platform_admin,
        filters,
        fetched: &fetched,
        inactive_days,
    });

    Ok(super::render_typed_page(
        &engine,
        "analytics-dashboard",
        &ctx,
        &user_ctx,
        &mkt_ctx,
    ))
}

// Why: a trends page defaults to a week, not the request log's 24h — daily
// buckets need more than one of them to read as a trend.
fn resolve_range(query: &AnalyticsDashboardQuery) -> TimeRange {
    let user_picked = query.preset.is_some() || (query.from.is_some() && query.to.is_some());
    if user_picked {
        parse_time_range(&TimeRangeQuery {
            from: query.from.clone(),
            to: query.to.clone(),
            preset: query.preset.clone(),
        })
    } else {
        preset_to_range(TimeRangePreset::Days7)
    }
}

// Why: `?org=` is honoured only for a platform admin — a customer's own
// administrator holds `admin` too, so trusting it would turn a URL edit into
// a read of another customer's usage.
fn resolve_scope(
    user_ctx: &UserContext,
    query: &AnalyticsDashboardQuery,
    own_org_slug: Option<&str>,
) -> SiteScope {
    let org_slug = if user_ctx.is_platform_admin {
        query
            .org
            .clone()
            .filter(|s| !s.is_empty())
            .map_or(OrgScope::AllOrganizations, OrgScope::Only)
    } else {
        // Why: An admin whose membership is missing scopes to the empty slug,
        // which matches nothing. Widening them to every organization would
        // reach the cross-customer view by having the least attachment.
        OrgScope::Only(own_org_slug.unwrap_or_default().to_owned())
    };
    SiteScope {
        org_slug,
        department: query.department.clone().filter(|s| !s.is_empty()),
        user_id: query.user_id.clone().filter(|u| !u.as_str().is_empty()),
    }
}

// Why: clamped rather than validated-and-rejected, because this arrives from a
// URL a person edits and a nonsensical value should show them the default
// window rather than an error page. The ceiling is a year because the window
// only ever asks "has this seat been used lately"; the floor is a day because
// a zero-day window would report every seat not in use this instant.
fn resolve_inactive_days(requested: Option<i32>) -> i32 {
    requested.map_or(seats::DEFAULT_INACTIVE_DAYS, |d| d.clamp(1, 365))
}

fn sort_links(query: &AnalyticsDashboardQuery) -> Vec<context::SortLinkView> {
    [
        ("Requests", "requests"),
        ("Cost", "cost"),
        ("Tokens", "tokens"),
        ("Last active", "last_active"),
    ]
    .into_iter()
    .map(|(label, key)| context::SortLinkView {
        label,
        href: urls::sort_url(query, key),
        is_active: LeaderboardSort::from_sort_param(query.sort.as_deref())
            == LeaderboardSort::from_sort_param(Some(key)),
    })
    .collect()
}

struct PageInput<'a> {
    query: &'a AnalyticsDashboardQuery,
    tab: DashboardTab,
    range: TimeRange,
    bucket: SeriesBucket,
    page: i64,
    is_platform_admin: bool,
    filters: FiltersView,
    fetched: &'a data::AnalyticsDashboardData,
    inactive_days: i32,
}

fn page_context(input: PageInput<'_>) -> AnalyticsDashboardContext {
    let PageInput {
        query,
        tab,
        range,
        bucket,
        page,
        is_platform_admin,
        filters,
        fetched,
        inactive_days,
    } = input;
    let weekly = bucket == SeriesBucket::Week;

    let leaderboard = view_tables::leaderboard_view(fetched, &range, query, page);
    let chips = urls::active_chips(query, is_platform_admin);
    let has_active_filters = !chips.is_empty();
    let meters = view::org_meters(&fetched.org_metrics);
    let own_meter = (tab == DashboardTab::Overview)
        .then(|| meters.first().cloned())
        .flatten();
    let burndown = view_spend::burndown_view(tab, fetched);
    let show_burndown_hint = tab == DashboardTab::Spend && burndown.is_none();
    let charts = view_models::overview_charts(fetched, query, &range, weekly);

    AnalyticsDashboardContext {
        page: "analytics-dashboard",
        title: "Analytics".to_owned(),
        time_range: view::time_range_view(query, &range),
        tabs: urls::tab_links(tab, query),
        is_overview: tab == DashboardTab::Overview,
        is_usage: tab == DashboardTab::Usage,
        is_seats: tab == DashboardTab::Seats,
        is_spend: tab == DashboardTab::Spend,
        is_code: tab == DashboardTab::Code,

        filters,
        chips,
        has_active_filters,
        clear_url: urls::clear_url(query),
        base_url: BASE_URL,

        kpis: view::kpi_strip(
            &fetched.kpis,
            fetched.wasted_seat_count,
            &range,
            query,
            (tab == DashboardTab::Overview).then_some(fetched.series.as_slice()),
        ),
        volume_chart: charts.volume,
        cost_chart: charts.cost,
        model_pie: charts.model_pie,
        model_cost_chart: charts.model_cost,
        own_meter,

        leaderboard,
        permissions: view_tables::permission_stats(&fetched.permissions),

        seat_summary: view_tables::seat_summaries(&fetched.org_metrics),
        wasted_seats: view_tables::wasted_seat_rows(&fetched.inactive_seats),
        has_wasted_seats: !fetched.inactive_seats.is_empty(),
        inactive_days,
        inactive_day_options: urls::inactive_day_links(query, inactive_days),

        has_spend_meters: !meters.is_empty(),
        spend_meters: meters,
        latency_link: "/admin/entities/requests".to_owned(),
        burndown,
        show_burndown_hint,
        has_budget_warnings: !fetched.budget_history.is_empty(),
        budget_warnings: view_spend::budget_warning_rows(&fetched.budget_history),
        fast_slow: view_spend::fast_slow(&fetched.latency),
        session_costs: view_spend::session_costs(&fetched.session_costs),

        commit_chart: view_code::commit_chart(&fetched.code_series, &range),
        loc_chart: view_code::loc_chart(&fetched.code_series, &range),
        code_frames: view_code::code_frames(&fetched.code_totals),
    }
}
