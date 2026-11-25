//! `GET /admin/analysis/reports/{id}/status` — the poll behind a report page's
//! loading widget.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use serde::Serialize;
use sqlx::PgPool;

use crate::error::AdminResult;
use crate::handlers::ssr::page::Page;

use super::{load_report, require_console};

// Why: what the loading widget polls: the row's state and, when it failed,
// why — as JSON, so the page reloads itself once the verdict exists.
#[derive(Debug, Serialize)]
pub(crate) struct ReportStatus {
    status: String,
    error: Option<String>,
    elapsed_ms: i64,
}

pub(crate) async fn status(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> AdminResult<Json<ReportStatus>> {
    require_console(&shell)?;
    let row = load_report(&pool, &id).await?;
    Ok(Json(ReportStatus {
        status: row.status.clone(),
        error: row.last_error.clone(),
        elapsed_ms: (chrono::Utc::now() - row.created_at).num_milliseconds(),
    }))
}
