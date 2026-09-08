//! `/admin/analytics` — every question about what the AI actually did.
//!
//! Six URL-driven tabs over one scope and one window. **Overview** puts the
//! whole instance on one screen: KPIs, request and cost trends, the model mix,
//! the top-user leaderboard, the latency split, anomalies and code impact.
//! **Models** is per-model gateway behaviour including route redirects and the
//! unrouted bucket. **Skills** is adoption, measured only. **Tools** is MCP
//! execution health. **Sessions** is client-reported session cost and rating.
//! **Cost** is the supplier bill, with a customer view that carries no
//! supplier figure at all.
//!
//! Scope and window live in the query string and are shared with the rest of
//! the AI-activity group through `js/services/scope.js`: `group`, `project`,
//! `user_id`, `preset`. Attribution is a separate claim about the same rows —
//! `Exclusive` by default, so container totals partition the instance; `?attr=
//! member` opts into the overlapping "who uses what" reading, and the page
//! says on screen that it overlaps.
//!
//! Console roles only. Every row drills into `/admin/requests` carrying the
//! dimension it names alongside the scope and window already on screen.

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::repositories::analytics::site::cost::ContainerAxis;
use crate::repositories::analytics::site::leaderboards::LeaderboardSort;
use crate::repositories::analytics::site::resolve_site_scope;
use crate::repositories::analytics::site::series::SeriesBucket;
use crate::repositories::scope::{Attribution, Scope, ScopeRequest};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use crate::util::time_range::{
    TimeRange, TimeRangePreset, TimeRangeQuery, parse_time_range, preset_to_range,
};

mod context;
mod context_overview;
mod context_tabs;
pub(crate) mod csv;
mod data;
mod data_tabs;
mod filters;
mod page;
mod tab_cost;
mod tab_models;
// Why: the overview's model board names models the way the Models tab does,
// so a prefix shortened here is shortened the same one click deeper.
pub(crate) use tab_models::{ms, qualifier, short_name};
mod tab_sessions;
mod tab_skills;
mod tab_tools;
mod urls;
mod urls_controls;
mod view;
mod view_code;
mod view_models;
mod view_spend;
mod view_tables;

use context::DashboardTab;
use page::{PageInput, page_context};

const BASE_URL: &str = "/admin/analytics";
const PAGE_SIZE: i64 = 50;

#[derive(Debug, Deserialize)]
pub(crate) struct AnalyticsDashboardQuery {
    pub tab: Option<String>,
    pub preset: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub bucket: Option<String>,
    pub group: Option<String>,
    pub project: Option<String>,
    pub user_id: Option<systemprompt::identifiers::UserId>,
    pub sort: Option<String>,
    pub page: Option<i64>,
    pub slo_ms: Option<i32>,
    // Why: `exclusive` (default) or `member` — how a person in more than one
    // container is counted.
    pub attr: Option<String>,
    // Why: Cost tab only — `internal` (default) or `customer`.
    pub audience: Option<String>,
    // Why: Cost tab only — `group` (default) or `project`.
    pub axis: Option<String>,
}

impl AnalyticsDashboardQuery {
    // Why: the container the reader named, most specific first. A user id is
    // the narrowest scope there is, so it wins over the project and group that
    // may still be riding along in the query string.
    pub(crate) fn scope(&self) -> Scope {
        if let Some(user_id) = self.user_id.clone().filter(|u| !u.as_str().is_empty()) {
            return Scope::User(user_id);
        }
        if let Some(project) = self.project.clone().filter(|p| !p.is_empty()) {
            return Scope::Project(project);
        }
        self.group
            .clone()
            .filter(|g| !g.is_empty())
            .map_or(Scope::All, Scope::Group)
    }

    pub(crate) fn attribution(&self) -> Attribution {
        if self.attr.as_deref() == Some("member") {
            Attribution::Member
        } else {
            Attribution::Exclusive
        }
    }

    pub(crate) fn is_internal_audience(&self) -> bool {
        self.audience.as_deref() != Some("customer")
    }

    pub(crate) fn container_axis(&self) -> ContainerAxis {
        if self.axis.as_deref() == Some("project") {
            ContainerAxis::Project
        } else {
            ContainerAxis::Group
        }
    }
}

pub(crate) async fn analytics_dashboard_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<AnalyticsDashboardQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let tab = DashboardTab::from_query(query.tab.as_deref());
    let bucket = SeriesBucket::from_bucket_param(query.bucket.as_deref());
    let sort = LeaderboardSort::from_sort_param(query.sort.as_deref());
    let range = resolve_range(&query);
    let page = query.page.unwrap_or(0).max(0);

    let request =
        ScopeRequest::from_query(&user_ctx, query.group.as_deref(), query.project.as_deref());
    let scope = resolve_site_scope(&pool, &query.scope(), query.attribution()).await?;
    let slo_ms = crate::repositories::analytics::site::latency::resolve_slo_ms(query.slo_ms);

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
            slo_ms,
            axis: query.container_axis(),
        },
    )
    .await;

    let filters = filters::build_filters(&pool, &user_ctx, &query, &request, bucket).await;
    let ctx = page_context(PageInput {
        query: &query,
        tab,
        range,
        bucket,
        page,
        filters,
        fetched: &fetched,
        slo_ms,
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
pub(super) fn resolve_range(query: &AnalyticsDashboardQuery) -> TimeRange {
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
