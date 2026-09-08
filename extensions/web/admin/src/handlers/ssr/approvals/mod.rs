//! `/admin/governance/approvals` — the calls a policy parked for a human.
//!
//! `require_approval` holds a tool call and blocks its caller on an
//! `approval_requests` row. This page is the other end of that rendezvous: the
//! queue is a live waiting room, not a log, so it leads with age and the
//! arguments the approver is actually authorising.
//!
//! Reading the queue is a console act; deciding one is an admin act. The page
//! renders for a project manager and the buttons do not, which is why the
//! decision goes through `POST /api/public/admin/approvals/{call_id}/…` on the
//! write tier rather than through a form on this route.

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::list_view::{PageWindow, Pagination, SelectOptionView};
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::governance::approvals::{
    ApprovalStats, get_approval_stats, list_approvals_paged,
};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

mod view;

const BASE_URL: &str = "/admin/governance/approvals";
const PAGE_SIZE: i64 = 50;
const STATUSES: [&str; 4] = ["pending", "approved", "denied", "expired"];

#[derive(Debug, Default, Deserialize)]
pub(crate) struct ApprovalsQuery {
    pub status: Option<String>,
    pub page: Option<i64>,
}

#[derive(Debug, Serialize)]
struct ApprovalKpiView {
    label: &'static str,
    value: String,
    sub: &'static str,
    tone: &'static str,
    href: String,
    active: bool,
    hint: &'static str,
}

#[derive(Debug, Serialize)]
struct StatusTab {
    label: String,
    href: String,
    is_active: bool,
}

#[derive(Debug, Serialize)]
struct ApprovalsPageContext {
    page: &'static str,
    title: &'static str,
    subtitle: &'static str,
    breadcrumbs: Vec<BreadcrumbView>,
    tabs: Vec<StatusTab>,
    kpis: Vec<ApprovalKpiView>,
    rows: Vec<view::ApprovalRowView>,
    has_rows: bool,
    row_count: String,
    pagination: Pagination,
    statuses: Vec<SelectOptionView>,
    can_decide: bool,
    base_url: &'static str,
}

pub(crate) async fn approvals_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<ApprovalsQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let status_filter = query
        .status
        .as_deref()
        .filter(|s| STATUSES.contains(s))
        .map(str::to_owned);
    let page = query.page.unwrap_or(0).max(0);

    let stats = get_approval_stats(&pool).await.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "approval stats failed");
        ApprovalStats::default()
    });
    let (rows, total) =
        list_approvals_paged(&pool, status_filter.as_deref(), PAGE_SIZE, page * PAGE_SIZE)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "approval queue failed");
                (Vec::new(), 0)
            });

    let shown = i64::try_from(rows.len()).unwrap_or(0);
    let ctx = ApprovalsPageContext {
        page: "governance-approvals",
        title: "Approvals",
        subtitle: "Tool calls held for a person. Each pending row is a caller still blocked.",
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::link("Governance", "/admin/governance"),
            BreadcrumbView::current("Approvals"),
        ],
        tabs: status_tabs(status_filter.as_deref()),
        kpis: kpis(&stats, status_filter.as_deref()),
        rows: view::rows(&rows),
        has_rows: !rows.is_empty(),
        row_count: format!("{total} requests"),
        pagination: pagination(page, total, shown, status_filter.as_deref()),
        statuses: status_options(status_filter.as_deref()),
        can_decide: user_ctx.is_admin,
        base_url: BASE_URL,
    };

    Ok(super::render_typed_page(
        &engine,
        "governance-approvals",
        &ctx,
        &user_ctx,
        &mkt_ctx,
    ))
}

fn url_for(status: Option<&str>, page: i64) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(status) = status {
        parts.push(format!("status={status}"));
    }
    if page > 0 {
        parts.push(format!("page={page}"));
    }
    if parts.is_empty() {
        BASE_URL.to_owned()
    } else {
        format!("{BASE_URL}?{}", parts.join("&"))
    }
}

fn status_tabs(active: Option<&str>) -> Vec<StatusTab> {
    let mut tabs = vec![StatusTab {
        label: "All".to_owned(),
        href: url_for(None, 0),
        is_active: active.is_none(),
    }];
    tabs.extend(STATUSES.iter().map(|status| StatusTab {
        label: view::title_case(status),
        href: url_for(Some(status), 0),
        is_active: active == Some(*status),
    }));
    tabs
}

// Why: the oldest pending age is the queue's service-level number and the only
// tile that is not a count — an approver arriving at the page needs to know
// whether anyone has been waiting past the point of usefulness.
fn kpis(stats: &ApprovalStats, active: Option<&str>) -> Vec<ApprovalKpiView> {
    vec![
        ApprovalKpiView {
            label: "Pending",
            value: stats.pending.to_string(),
            sub: "callers blocked now",
            tone: "warn",
            href: url_for(Some("pending"), 0),
            active: active == Some("pending"),
            hint: "Rows still open and not yet past their expiry",
        },
        ApprovalKpiView {
            label: "Oldest wait",
            value: format!("{}m", stats.oldest_pending_minutes),
            sub: "longest open request",
            tone: if stats.oldest_pending_minutes > 30 {
                "err"
            } else {
                "ok"
            },
            href: url_for(Some("pending"), 0),
            active: false,
            hint: "Minutes since the oldest still-pending call was parked",
        },
        ApprovalKpiView {
            label: "Approved",
            value: stats.approved.to_string(),
            sub: "let through by a person",
            tone: "ok",
            href: url_for(Some("approved"), 0),
            active: active == Some("approved"),
            hint: "Calls a named approver released",
        },
        ApprovalKpiView {
            label: "Denied",
            value: stats.denied.to_string(),
            sub: "refused by a person",
            tone: "err",
            href: url_for(Some("denied"), 0),
            active: active == Some("denied"),
            hint: "Calls a named approver refused",
        },
        ApprovalKpiView {
            label: "Expired",
            value: stats.expired.to_string(),
            sub: "nobody answered",
            tone: "",
            href: url_for(Some("expired"), 0),
            active: active == Some("expired"),
            hint: "Requests that timed out unanswered — a decision by omission",
        },
    ]
}

fn pagination(page: i64, total: i64, shown: i64, status: Option<&str>) -> Pagination {
    let window = PageWindow::new(page, PAGE_SIZE, total, shown, "requests");
    let (first_row, last_row) = window.bounds();
    Pagination {
        current_page: page + 1,
        total_pages: window.total_pages,
        first_row,
        last_row,
        total_rows: total,
        noun: "requests",
        has_prev: page > 0,
        has_next: page + 1 < window.total_pages,
        prev_url: (page > 0).then(|| url_for(status, page - 1)),
        next_url: (page + 1 < window.total_pages).then(|| url_for(status, page + 1)),
    }
}

fn status_options(active: Option<&str>) -> Vec<SelectOptionView> {
    let mut out = vec![SelectOptionView {
        value: String::new(),
        label: "All statuses".to_owned(),
        selected: active.is_none(),
    }];
    out.extend(STATUSES.iter().map(|status| SelectOptionView {
        value: (*status).to_owned(),
        label: view::title_case(status),
        selected: active == Some(*status),
    }));
    out
}
