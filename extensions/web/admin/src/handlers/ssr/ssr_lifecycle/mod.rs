//! `/admin/lifecycle` — what the retention jobs measured, archived and found.
//!
//! Three ledgers, all written by the `retention_*` jobs in the web jobs
//! crate: the latest daily measurement per managed table with its week-on-
//! week growth, the weekly and monthly archives under `storage/exports/`
//! with their digests and a download link, and the last monthly health
//! report's ranked findings. Deletion itself is core's `database_cleanup`;
//! its windows and last run are shown on `/admin/configuration`.

mod view;

use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::config::{AppPaths, ProfileBootstrap};
use systemprompt::loader::ServicesBootstrap;

use self::view::{ArchivePeriodView, LifecycleFindingView, MeasureView, build_view};
use super::configuration::retention::{RetentionView, retention_view};
use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::lifecycle::{
    find_latest_health_report, list_latest_retention_runs, list_retention_archives,
};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

#[derive(Debug, Serialize)]
struct LifecyclePageData {
    page: &'static str,
    title: &'static str,
    breadcrumbs: Vec<BreadcrumbView>,
    configuration_url: &'static str,
    measures: Vec<MeasureView>,
    // Why: handlebars-rust has no `.length`, so the KPI takes the count as a field.
    measures_count: usize,
    periods: Vec<ArchivePeriodView>,
    archive_count: usize,
    health_run_at: Option<String>,
    findings: Vec<LifecycleFindingView>,
    findings_p1: i32,
    findings_p2: i32,
    findings_p3: i32,
    findings_p1_tone: &'static str,
    findings_p2_tone: &'static str,
    retention: RetentionView,
}

pub(crate) async fn lifecycle_page(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let profile = ProfileBootstrap::get().map_err(AdminError::internal)?;
    let ai_history_days = ServicesBootstrap::get().map_or(30, |s| s.ai.history.retention_days);
    let retention = retention_view(&pool, &profile.retention, ai_history_days).await?;
    let measures = list_latest_retention_runs(&pool).await?;
    let archives = list_retention_archives(&pool).await?;
    let health = find_latest_health_report(&pool).await?;
    let view = build_view(&measures, &archives, health.as_ref());
    let page = LifecyclePageData {
        page: "lifecycle",
        title: "Data lifecycle",
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::link("Configuration", "/admin/configuration"),
            BreadcrumbView::current("Data lifecycle"),
        ],
        configuration_url: "/admin/configuration",
        archive_count: archives.len(),
        measures_count: view.measures.len(),
        measures: view.measures,
        periods: view.periods,
        health_run_at: view.health_run_at,
        findings: view.findings,
        findings_p1: view.counts.0,
        findings_p2: view.counts.1,
        findings_p3: view.counts.2,
        findings_p1_tone: if view.counts.0 > 0 { "err" } else { "neutral" },
        findings_p2_tone: if view.counts.1 > 0 { "warn" } else { "neutral" },
        retention,
    };
    Ok(super::render_typed_page(
        &engine,
        "lifecycle",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}

// Why: the three path segments are validated against the exact shapes the
// jobs write (`weekly|monthly`, `yyyy-Www|yyyy-mm`, `<table>.jsonl.gz` or
// `manifest.json`) before any filesystem read, so the route can never be
// steered outside the exports directory.
pub(crate) async fn lifecycle_archive_download(
    Extension(user_ctx): Extension<UserContext>,
    Path((tier, period, file)): Path<(String, String, String)>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    if !view::valid_tier(&tier) || !view::valid_period(&period) || !view::valid_file(&file) {
        return Err(AdminError::NotFound("No such archive".to_owned()).into());
    }
    let profile = ProfileBootstrap::get().map_err(AdminError::internal)?;
    let paths = AppPaths::from_profile(
        &profile.paths,
        profile.path_resolution(),
        systemprompt::loader::ServicesRootBootstrap::get().map(|root| root.path.as_path()),
    )
    .map_err(AdminError::internal)?;
    let path = paths
        .storage()
        .exports()
        .join(&tier)
        .join(&period)
        .join(&file);
    // Why: the cause is logged in full and the client is told only that the
    // archive is not there. The path is a server-side detail, and a read that
    // failed for any other reason is still nothing this caller can fetch.
    let bytes = tokio::fs::read(&path).await.map_err(|error| {
        tracing::warn!(path = %path.display(), %error, "lifecycle archive unreadable");
        AdminError::NotFound("No such archive".to_owned())
    })?;
    let content_type = if std::path::Path::new(&file)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
    {
        "application/json"
    } else {
        "application/gzip"
    };
    Ok((
        [
            (header::CONTENT_TYPE, content_type.to_owned()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{tier}-{period}-{file}\""),
            ),
        ],
        bytes,
    )
        .into_response())
}
