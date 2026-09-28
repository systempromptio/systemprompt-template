//! `/admin/governance/secrets` — every recorded touch of a stored credential.
//!
//! Secrets are sealed with a per-user key and plaintext never reaches the
//! database, so this append-only table is the entire record of what happened to
//! a credential. The question it answers is not "what is stored" but "who read
//! it, and were they its owner" — which is why the owner and the actor are two
//! columns and the count of rows where they differ is a KPI.
//!
//! Until now the trail was reachable only through the per-plugin JSON API. The
//! query here is the console-wide form of that read, paged and exportable.

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult, AdminResult};
use crate::handlers::ssr::list_view::{PageWindow, Pagination, SelectOptionView, TimeRangeContext};
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::governance::secret_audit_log::{
    SecretAuditFilter, SecretAuditRow, SecretAuditStats, get_secret_audit_stats,
    list_secret_audit_actions, list_secret_audit_paged,
};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use crate::util::time_range::{TimeRange, TimeRangeQuery, parse_time_range};

mod context;
mod view;

use context::{SecretAuditKpiView, action_options, kpis, url_for};

const BASE_URL: &str = "/admin/governance/secrets";
const PAGE_SIZE: i64 = 50;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct SecretsQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub preset: Option<String>,
    pub action: Option<String>,
    pub q: Option<String>,
    pub page: Option<i64>,
}

#[derive(Debug, Serialize)]
struct SecretsPageContext {
    page: &'static str,
    title: &'static str,
    subtitle: &'static str,
    breadcrumbs: Vec<BreadcrumbView>,
    time_range: TimeRangeContext,
    kpis: Vec<SecretAuditKpiView>,
    rows: Vec<view::SecretAuditRowView>,
    has_rows: bool,
    row_count: String,
    pagination: Pagination,
    actions: Vec<SelectOptionView>,
    search: String,
    export: crate::export::ExportView,
    clear_url: &'static str,
    has_filters: bool,
    base_url: &'static str,
}

fn require_console(user_ctx: &UserContext) -> Result<(), AdminError> {
    if user_ctx.is_console {
        return Ok(());
    }
    Err(AdminError::Forbidden("Admin access required.".to_owned()))
}

fn range_of(query: &SecretsQuery) -> TimeRange {
    parse_time_range(&TimeRangeQuery {
        from: query.from.clone(),
        to: query.to.clone(),
        preset: query.preset.clone(),
    })
}

fn filter_of(query: &SecretsQuery) -> SecretAuditFilter {
    SecretAuditFilter {
        action: query
            .action
            .as_deref()
            .filter(|a| !a.is_empty())
            .map(str::to_owned),
        search: query
            .q
            .as_deref()
            .filter(|q| !q.trim().is_empty())
            .map(str::to_owned),
    }
}

fn time_range_context(query: &SecretsQuery, range: TimeRange) -> TimeRangeContext {
    TimeRangeContext {
        preset: query.preset.clone().unwrap_or_else(|| "7d".to_owned()),
        from: range.from.format("%Y-%m-%dT%H:%M").to_string(),
        to: range.to.format("%Y-%m-%dT%H:%M").to_string(),
        base_url: BASE_URL,
        query: "",
        rejected: range.rejected_bounds,
    }
}

fn pagination(
    query: &SecretsQuery,
    action: Option<&str>,
    page: i64,
    window: PageWindow,
) -> Pagination {
    let (first_row, last_row) = window.bounds();
    Pagination {
        current_page: page + 1,
        total_pages: window.total_pages,
        first_row,
        last_row,
        total_rows: window.total_rows,
        noun: "entries",
        has_prev: page > 0,
        has_next: page + 1 < window.total_pages,
        prev_url: (page > 0).then(|| url_for(query, action, page - 1)),
        next_url: (page + 1 < window.total_pages).then(|| url_for(query, action, page + 1)),
    }
}

pub(crate) async fn secrets_audit_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<SecretsQuery>,
) -> AdminHtmlResult<Response> {
    require_console(&user_ctx)?;

    let range = range_of(&query);
    let filter = filter_of(&query);
    let page = query.page.unwrap_or(0).max(0);

    let stats = get_secret_audit_stats(&pool, range)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "secret audit stats failed");
            SecretAuditStats::default()
        });
    let (rows, total) = list_secret_audit_paged(&pool, range, &filter, PAGE_SIZE, page * PAGE_SIZE)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "secret audit log failed");
            (Vec::new(), 0)
        });
    let actions = list_secret_audit_actions(&pool, range)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "secrets audit: action listing failed"))
        .unwrap_or_default();

    let shown = i64::try_from(rows.len()).unwrap_or(0);
    let window = PageWindow::new(page, PAGE_SIZE, total, shown, "entries");

    let ctx = SecretsPageContext {
        page: "governance-secrets",
        title: "Secrets audit",
        subtitle: "Every recorded action against a stored credential: who read what, and whether they owned it.",
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::link("Governance", "/admin/governance"),
            BreadcrumbView::current("Secrets audit"),
        ],
        time_range: time_range_context(&query, range),
        kpis: kpis(&stats, &query),
        rows: view::rows(&rows),
        has_rows: !rows.is_empty(),
        row_count: format!("{total} entries"),
        pagination: pagination(&query, filter.action.as_deref(), page, window),
        actions: action_options(&actions, filter.action.as_deref()),
        search: query.q.clone().unwrap_or_default(),
        export: crate::export::ExportView::single(
            "governance-secrets",
            &crate::export::view::query_string(&[
                ("preset", query.preset.as_deref()),
                ("from", query.from.as_deref()),
                ("to", query.to.as_deref()),
                ("action", query.action.as_deref()),
                ("q", query.q.as_deref()),
            ]),
        ),
        clear_url: BASE_URL,
        has_filters: filter.action.is_some() || filter.search.is_some(),
        base_url: BASE_URL,
    };

    Ok(super::render_typed_page(
        &engine,
        "governance-secrets",
        &ctx,
        &user_ctx,
        &mkt_ctx,
    ))
}

// Why: the export reads the same window and filter the page shows, through
// the page's own query type.
pub(crate) async fn export_rows(
    pool: &PgPool,
    user_ctx: &UserContext,
    query: SecretsQuery,
    range: TimeRange,
    limit: i64,
) -> AdminResult<(Vec<SecretAuditRow>, i64)> {
    require_console(user_ctx)?;
    Ok(list_secret_audit_paged(pool, range, &filter_of(&query), limit, 0).await?)
}
