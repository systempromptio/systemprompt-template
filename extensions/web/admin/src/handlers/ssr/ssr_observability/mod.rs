//! `/admin/system/observability` — where this instance ships its telemetry
//! and how far behind the shipment is.
//!
//! The exporter is core's `otlp_export` job; the profile's
//! `observability.otlp` block is what it reads. This page shows that block
//! beside the job's own ledger (`otlp_export_state`: one cursor per signal,
//! lag, last success, last error, counters) and offers two actions: **Export
//! now** runs the job's batch out of turn through core's `otlp_export_now`,
//! and **Test connection** posts an empty envelope to the collector
//! ([`probe`]). Neither action exports anything of its own; the outcome comes
//! back as a query string and is read once, on the redirect.

mod probe;

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::http::HeaderMap;
use axum::response::{Redirect, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::manifest::profile::OtlpExportConfig;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::shared::require_write_origin;
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::observability::view::{ObservabilityView, build_view};
use crate::repositories::observability::{find_declared_export, list_export_states};
use crate::routes::managed_state::ManagedState;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

pub(crate) const PAGE_URL: &str = "/admin/system/observability";
const DOCS_URL: &str = "/documentation/enterprise-audit-observability";
const DETAIL_MAX: usize = 240;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct ObservabilityQuery {
    outcome: Option<String>,
    detail: Option<String>,
}

// Why: the notice the redirect carries — a fixed code picks the tone and
// title, the detail is the one free-text line and is printed escaped.
#[derive(Debug, Clone, Serialize)]
struct OutcomeView {
    tone: &'static str,
    title: &'static str,
    detail: String,
}

impl OutcomeView {
    fn from_query(query: &ObservabilityQuery) -> Option<Self> {
        let (tone, title) = match query.outcome.as_deref()? {
            "exported" => ("ok", "Export ran"),
            "export_failed" => ("err", "Export failed"),
            "probe_ok" => ("ok", "Collector reachable"),
            "probe_failed" => ("err", "Collector refused the probe"),
            "unconfigured" => ("warn", "Nothing to export"),
            _ => return None,
        };
        Some(Self {
            tone,
            title,
            detail: query
                .detail
                .as_deref()
                .unwrap_or_default()
                .chars()
                .take(DETAIL_MAX)
                .collect(),
        })
    }
}

#[derive(Debug, Serialize)]
struct ObservabilityPageData {
    page: &'static str,
    title: &'static str,
    can_write: bool,
    breadcrumbs: Vec<BreadcrumbView>,
    docs_url: &'static str,
    sync_url: &'static str,
    export_url: String,
    test_url: String,
    job_name: &'static str,
    outcome: Option<OutcomeView>,
    #[serde(flatten)]
    view: ObservabilityView,
}

pub(crate) async fn observability_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Extension(state): Extension<Arc<ManagedState>>,
    Query(query): Query<ObservabilityQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let config = find_declared_export()?;
    let states = list_export_states(&state.otlp_export).await?;
    let page = ObservabilityPageData {
        page: "system-observability",
        title: "Observability",
        can_write: user_ctx.is_admin,
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::link("Sync", "/admin/sync"),
            BreadcrumbView::current("Observability"),
        ],
        docs_url: DOCS_URL,
        sync_url: "/admin/sync",
        export_url: format!("{PAGE_URL}/export"),
        test_url: format!("{PAGE_URL}/test"),
        job_name: systemprompt::scheduler::jobs::otlp_export::JOB_NAME,
        outcome: OutcomeView::from_query(&query),
        view: build_view(config.as_ref(), &states),
    };
    Ok(super::render_typed_page(
        &engine,
        "system-observability",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}

fn redirect(outcome: &str, detail: &str) -> Redirect {
    let detail: String = detail.chars().take(DETAIL_MAX).collect();
    Redirect::to(&format!(
        "{PAGE_URL}?outcome={outcome}&detail={}",
        urlencoding::encode(&detail)
    ))
}

fn write_gate(user_ctx: &UserContext, headers: &HeaderMap) -> AdminHtmlResult<()> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Administrator access required".to_owned()).into());
    }
    require_write_origin(headers)?;
    Ok(())
}

fn configured_or_redirect(config: Option<OtlpExportConfig>) -> Result<OtlpExportConfig, Redirect> {
    config.ok_or_else(|| {
        redirect(
            "unconfigured",
            "The profile declares no observability.otlp block.",
        )
    })
}

pub(crate) async fn export_now(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
) -> AdminHtmlResult<Redirect> {
    write_gate(&user_ctx, &headers)?;
    let config = match configured_or_redirect(find_declared_export()?) {
        Ok(config) => config,
        Err(redirect) => return Ok(redirect),
    };
    let instance_id = systemprompt::manifest::Config::get()
        .ok()
        .map(|c| c.instance_id.clone());
    let report = systemprompt::scheduler::otlp_export_now(&pool, &config, instance_id.as_ref())
        .await
        .map_err(AdminError::internal)?;
    tracing::info!(
        requested_by = %user_ctx.user_id,
        rows = report.rows(),
        failed = report.failed(),
        "OTLP export run from the console"
    );
    let failed: Vec<String> = report
        .signals
        .iter()
        .filter_map(|s| s.error.as_ref().map(|e| format!("{}: {e}", s.signal)))
        .collect();
    Ok(if failed.is_empty() {
        redirect(
            "exported",
            &format!(
                "{} rows across {} signal(s).",
                report.rows(),
                report.signals.len()
            ),
        )
    } else {
        redirect("export_failed", &failed.join("; "))
    })
}

pub(crate) async fn test_connection(
    Extension(user_ctx): Extension<UserContext>,
    headers: HeaderMap,
) -> AdminHtmlResult<Redirect> {
    write_gate(&user_ctx, &headers)?;
    let config = match configured_or_redirect(find_declared_export()?) {
        Ok(config) => config,
        Err(redirect) => return Ok(redirect),
    };
    Ok(match probe::probe(&config).await {
        Ok(outcome) => {
            tracing::info!(requested_by = %user_ctx.user_id, url = %outcome.url, status = %outcome.status, "OTLP collector probe succeeded");
            redirect(
                "probe_ok",
                &format!(
                    "{} answered {} in {} ms.",
                    outcome.url, outcome.status, outcome.elapsed_ms
                ),
            )
        },
        Err(error) => {
            tracing::warn!(requested_by = %user_ctx.user_id, error = %error, "OTLP collector probe failed");
            redirect("probe_failed", &error.to_string())
        },
    })
}
