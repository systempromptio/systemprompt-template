//! `/admin/reports/customer` — the sendable month-end usage report.
//!
//! One organization, one calendar month: seats, tokens, who used them, which
//! department they sat in, which models they reached, and the licence fee the
//! month is billed at. Nothing about what serving them cost us — that is the
//! internal report, and the two share no query.
//!
//! Admin-only, and scoped. A platform admin may name any organization with
//! `?org=<slug>`; anyone else gets their own and has no way to ask for
//! another's. The page carries a print action, so "send it to the customer"
//! is a PDF the operator produces from the browser rather than a second
//! rendering path that could drift from this one.

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::repositories::organizations::crud;
use crate::repositories::reports::customer;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use crate::util::month_range::{MonthQuery, MonthRange, list_month_options, parse_month_range};

mod context;
mod view;

use context::{OrgOption, ReportCustomerContext};

const BASE_URL: &str = "/admin/reports/customer";

#[derive(Debug, Deserialize)]
pub(crate) struct CustomerReportQuery {
    pub month: Option<String>,
    pub org: Option<String>,
}

pub(crate) async fn report_customer_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<CustomerReportQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let month = parse_month_range(&MonthQuery {
        month: query.month.clone(),
    });
    let slug = resolve_slug(&pool, &user_ctx, query.org.as_deref()).await?;
    let Some(org) = crud::find_organization_by_slug(&pool, &slug).await? else {
        return Err(AdminError::NotFound(format!("No organization with slug '{slug}'.")).into());
    };

    let summary =
        customer::find_customer_month_summary(&pool, &org.id, month.from, month.to).await?;
    let users = customer::list_customer_month_users(&pool, &org.id, month.from, month.to).await?;
    let departments =
        customer::list_customer_month_departments(&pool, &org.id, month.from, month.to).await?;
    let models = customer::list_customer_month_models(&pool, &org.id, month.from, month.to).await?;

    let org_options = if user_ctx.is_platform_admin {
        org_options(&pool, &slug).await?
    } else {
        Vec::new()
    };

    let ctx = build_context(BuildInput {
        org: &org,
        month: &month,
        summary: summary.as_ref(),
        users: &users,
        departments: &departments,
        models: &models,
        is_platform_admin: user_ctx.is_platform_admin,
        org_options,
    });

    Ok(super::render_typed_page(
        &engine,
        "report-customer",
        &ctx,
        &user_ctx,
        &mkt_ctx,
    ))
}

// Why: `?org=` is honoured only for a platform admin — a customer's own
// administrator holds `admin` too, so trusting it would turn a URL edit into
// a read of another customer's usage.
async fn resolve_slug(
    pool: &PgPool,
    user_ctx: &UserContext,
    requested: Option<&str>,
) -> Result<String, AdminError> {
    if user_ctx.is_platform_admin
        && let Some(slug) = requested.filter(|s| !s.is_empty())
    {
        return Ok(slug.to_owned());
    }
    crud::find_organization_for_user(pool, &user_ctx.user_id)
        .await?
        .ok_or_else(|| {
            AdminError::NotFound(
                "You are not a member of an organization, so there is no usage to report."
                    .to_owned(),
            )
        })
}

async fn org_options(pool: &PgPool, selected: &str) -> Result<Vec<OrgOption>, AdminError> {
    Ok(crud::list_organizations(pool)
        .await?
        .into_iter()
        .map(|o| OrgOption {
            selected: o.slug == selected,
            slug: o.slug,
            name: o.name,
        })
        .collect())
}

struct BuildInput<'a> {
    org: &'a crud::OrganizationSummary,
    month: &'a MonthRange,
    summary: Option<&'a customer::CustomerMonthSummary>,
    users: &'a [customer::CustomerUserUsage],
    departments: &'a [customer::CustomerDepartmentUsage],
    models: &'a [customer::CustomerModelUsage],
    is_platform_admin: bool,
    org_options: Vec<OrgOption>,
}

fn build_context(input: BuildInput<'_>) -> ReportCustomerContext {
    let BuildInput {
        org,
        month,
        summary,
        users,
        departments,
        models,
        is_platform_admin,
        org_options,
    } = input;

    let price = summary.map_or(0, |s| s.price_microdollars);
    let user_views = view::user_views(users);
    let department_views = view::department_views(departments);
    let model_views = view::model_views(models);
    let next = month.next();

    ReportCustomerContext {
        page: "report-customer",
        title: format!("{} — {}", org.name, month.label),
        org_name: org.name.clone(),
        org_slug: org.slug.clone(),
        plan_name: org.plan_name.clone(),
        plan_price_microdollars: price,
        has_price: price > 0,
        month_key: month.key.clone(),
        month_label: month.label.clone(),
        month_complete: month.is_complete,
        months: list_month_options(month),
        prev_url: month_url(&org.slug, &month.previous().key),
        next_url: next.as_ref().map(|m| month_url(&org.slug, &m.key)),
        has_next: next.is_some(),
        base_url: BASE_URL,
        generated_at: chrono::Utc::now().format("%d %B %Y").to_string(),
        is_platform_admin,
        org_options,
        summary: summary.map_or_else(view::empty_summary_view, view::summary_view),
        users: user_views,
        departments: department_views,
        models: model_views,
    }
}

fn month_url(slug: &str, key: &str) -> String {
    format!("{BASE_URL}?org={slug}&month={key}")
}
