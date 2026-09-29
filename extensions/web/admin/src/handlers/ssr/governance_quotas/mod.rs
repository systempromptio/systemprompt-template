//! `/admin/governance/quotas` — usage against ceiling, per subject, for
//! every quota window the gateway is enforcing right now.
//!
//! The windows are the effective spec (core's merge of the enabled
//! policy rows); the usage is the one bucket per subject the gateway
//! reserves against in the current period. Under `quota_mode: warn` a
//! bucket past its ceiling is a warning the request survived, and the page
//! says so rather than printing a red number that never refused anything.

use std::sync::Arc;

use axum::extract::State;
use axum::response::Response;
use serde::Serialize;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::format::relative_time;
use crate::handlers::ssr::page::Page;
use crate::handlers::ssr::people_view::format_usd;
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::gateway_policies::month_window::{is_month_window, window_label};
use crate::repositories::gateway_policies::rows::{effective_spec, list_policies};
use crate::repositories::gateway_policies::usage::{get_window_usage, headroom};

#[derive(Debug, Serialize)]
struct SubjectRow {
    subject_id: String,
    subject_url: Option<String>,
    requests: i64,
    input_tokens: i64,
    output_tokens: i64,
    cost: String,
    percent: u32,
    bar_percent: u32,
    ceiling: &'static str,
    tone: &'static str,
    breached: bool,
    updated: String,
}

#[derive(Debug, Serialize)]
struct WindowCard {
    subject_kind: String,
    label: String,
    is_month: bool,
    window_start: String,
    window_end: String,
    max_requests: Option<i64>,
    max_input_tokens: Option<i64>,
    max_output_tokens: Option<i64>,
    max_cost: Option<String>,
    rows: Vec<SubjectRow>,
    row_count: usize,
    breached: usize,
}

#[derive(Debug, Serialize)]
struct QuotasPageData {
    page: &'static str,
    title: &'static str,
    breadcrumbs: Vec<BreadcrumbView>,
    quota_warn: bool,
    windows: Vec<WindowCard>,
    has_windows: bool,
    editor_url: &'static str,
    docs_url: &'static str,
}

fn subject_url(kind: &str, id: &str) -> Option<String> {
    match kind {
        "user" => Some(format!("/admin/users/{id}")),
        "group" => Some(format!("/admin/groups/{id}")),
        "project" => Some(format!("/admin/projects/{id}")),
        _ => None,
    }
}

const fn tone(percent: u32, breached: bool) -> &'static str {
    if breached {
        "err"
    } else if percent >= 80 {
        "warn"
    } else {
        "ok"
    }
}

pub(crate) async fn governance_quotas_page(
    page: Page,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Response> {
    if !page.user.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let now = chrono::Utc::now();
    let today = now.date_naive();
    let rows = list_policies(&pool).await.map_err(AdminError::Database)?;
    let spec = effective_spec(&rows);

    let mut windows = Vec::with_capacity(spec.quota_windows.len());
    for window in &spec.quota_windows {
        let usage = get_window_usage(&pool, window, now)
            .await
            .map_err(AdminError::Database)?;
        let rows: Vec<SubjectRow> = usage
            .subjects
            .iter()
            .map(|u| {
                let h = headroom(window, u);
                SubjectRow {
                    subject_url: subject_url(&u.subject_kind, &u.subject_id),
                    subject_id: u.subject_id.clone(),
                    requests: u.requests,
                    input_tokens: u.input_tokens,
                    output_tokens: u.output_tokens,
                    cost: format_usd(u.cost_microdollars),
                    percent: h.percent,
                    bar_percent: h.percent.min(100),
                    ceiling: h.ceiling,
                    tone: tone(h.percent, h.breached),
                    breached: h.breached,
                    updated: relative_time(u.updated_at),
                }
            })
            .collect();
        windows.push(WindowCard {
            subject_kind: window.subject.clone(),
            label: window_label(window.window_seconds, today),
            is_month: is_month_window(window.window_seconds),
            window_start: usage.window_start.to_rfc3339(),
            window_end: usage.window_end.to_rfc3339(),
            max_requests: window.max_requests,
            max_input_tokens: window.max_input_tokens,
            max_output_tokens: window.max_output_tokens,
            max_cost: window.max_cost_microdollars.map(format_usd),
            breached: rows.iter().filter(|r| r.breached).count(),
            row_count: rows.len(),
            rows,
        });
    }

    let data = QuotasPageData {
        page: "governance-quotas",
        title: "Quotas",
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::link("Governance", "/admin/governance"),
            BreadcrumbView::current("Quotas"),
        ],
        quota_warn: spec.quota_mode.is_warn(),
        has_windows: !windows.is_empty(),
        windows,
        editor_url: "/admin/gateway/policies",
        docs_url: "/documentation/services-sync",
    };
    Ok(crate::handlers::ssr::render_typed_page(
        &page.engine,
        "governance-quotas",
        &data,
        &page.user,
        &page.marketplace,
    ))
}
