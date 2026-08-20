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
use crate::handlers::ssr::list_view::PageWindow;
use crate::repositories::analytics::site::SiteScope;
use crate::repositories::analytics::site::leaderboards::LeaderboardSort;
use crate::repositories::analytics::site::series::SeriesBucket;
use crate::repositories::organizations::crud;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use crate::util::time_range::{
    TimeRange, TimeRangePreset, TimeRangeQuery, parse_time_range, preset_to_range,
};

mod context;
mod data;
mod filters;
mod urls;
mod view;
mod view_code;
mod view_tables;

use context::{AnalyticsDashboardContext, DashboardTab, FiltersView, LeaderboardView};

const BASE_URL: &str = "/admin/analytics";
const PAGE_SIZE: i64 = 50;

#[derive(Debug, Deserialize)]
pub(crate) struct AnalyticsDashboardQuery {
    pub tab: Option<String>,
    pub preset: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub bucket: Option<String>,
    /// Honoured for platform admins only.
    pub org: Option<String>,
    pub department: Option<String>,
    pub user_id: Option<systemprompt::identifiers::UserId>,
    pub sort: Option<String>,
    pub page: Option<i64>,
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
        query.org.clone().filter(|s| !s.is_empty())
    } else {
        own_org_slug.map(str::to_owned)
    };
    SiteScope {
        org_slug,
        department: query.department.clone().filter(|s| !s.is_empty()),
        user_id: query.user_id.clone().filter(|u| !u.as_str().is_empty()),
    }
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
    } = input;
    let weekly = bucket == SeriesBucket::Week;

    let leaderboard_rows = view_tables::leaderboard_rows(&fetched.leaderboard, &range, query);
    let pagination = urls::build_pagination(
        query,
        PageWindow::new(
            page,
            PAGE_SIZE,
            fetched.leaderboard_total,
            i64::try_from(fetched.leaderboard.len()).unwrap_or(PAGE_SIZE),
            "users",
        ),
    );
    let sort_links = sort_links(query);
    let chips = urls::active_chips(query, is_platform_admin);
    let has_active_filters = !chips.is_empty();
    let meters = view::org_meters(&fetched.org_metrics);
    let own_meter = (tab == DashboardTab::Overview)
        .then(|| meters.first().cloned())
        .flatten();

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

        kpis: view::kpi_strip(&fetched.kpis, fetched.wasted_seat_count, &range, query),
        volume_chart: view::volume_chart(&fetched.series, &range, weekly),
        cost_chart: view::spend_chart(&fetched.series, &range, weekly),
        model_pie: view::model_pie(&fetched.models, query),
        own_meter,

        leaderboard: LeaderboardView {
            has_rows: !leaderboard_rows.is_empty(),
            rows: leaderboard_rows,
            sort_links,
            pagination,
        },
        permissions: view_tables::permission_stats(&fetched.permissions),

        seat_summary: view_tables::seat_summaries(&fetched.org_metrics),
        wasted_seats: view_tables::wasted_seat_rows(&fetched.inactive_seats),
        has_wasted_seats: !fetched.inactive_seats.is_empty(),

        has_spend_meters: !meters.is_empty(),
        spend_meters: meters,
        latency_link: "/admin/entities/requests".to_owned(),

        commit_chart: view_code::commit_chart(&fetched.code_series, &range),
        loc_chart: view_code::loc_chart(&fetched.code_series, &range),
        code_frames: view_code::code_frames(&fetched.code_totals),
    }
}
