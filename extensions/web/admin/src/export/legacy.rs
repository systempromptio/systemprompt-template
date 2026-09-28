//! The per-page CSV URLs that predate the export surface.
//!
//! `/admin/requests.csv`, `/admin/analytics/cost.csv`,
//! `/admin/governance/{warnings,secrets}.csv` and
//! `/admin/reports/{customer,internal}.csv` stay mounted because bookmarks
//! and the finance hand-off fetch them. Each one names the dataset that now
//! answers it and is served by the same handler as `/admin/export/{dataset}`
//! with `format=csv`, so the old URL and the dialog can never disagree.

use std::sync::Arc;

use axum::extract::{Extension, OriginalUri, State};
use axum::http::Uri;
use axum::response::Response;
use sqlx::PgPool;

use super::handler::file_response;
use crate::error::{AdminError, AdminResult};
use crate::types::UserContext;

fn param<'a>(uri: &'a Uri, name: &str) -> Option<std::borrow::Cow<'a, str>> {
    url::form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
        .find(|(k, _)| k == name)
        .map(|(_, v)| v)
}

// Why: the old query string carries the page's own filters, which every
// dataset reads back through the page's query type; only the path and the
// format change.
async fn serve(id: &str, uri: &Uri, pool: &PgPool, user: &UserContext) -> AdminResult<Response> {
    let query = uri.query().unwrap_or_default();
    let sep = if query.is_empty() { "" } else { "&" };
    let rewritten: Uri = format!("/admin/export/{id}?{query}{sep}format=csv")
        .parse()
        // Why: lint-ok: error-adapt — `InvalidUri` is variant-less
        .map_err(|e| AdminError::BadRequest(format!("Unreadable export URL: {e}")))?;
    file_response(id, &rewritten, pool, user).await
}

pub(crate) async fn requests_csv(
    Extension(user): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    OriginalUri(uri): OriginalUri,
) -> AdminResult<Response> {
    serve("requests", &uri, &pool, &user).await
}

// Why: two audiences, two tables — the internal file carries provider cost,
// the customer file consumption and no cost column at all.
pub(crate) async fn cost_csv(
    Extension(user): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    OriginalUri(uri): OriginalUri,
) -> AdminResult<Response> {
    let id = if param(&uri, "audience").as_deref() == Some("internal") {
        "analytics-cost-providers"
    } else {
        "analytics-cost-containers"
    };
    serve(id, &uri, &pool, &user).await
}

pub(crate) async fn governance_csv(
    Extension(user): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    OriginalUri(uri): OriginalUri,
) -> AdminResult<Response> {
    serve("governance-decisions", &uri, &pool, &user).await
}

pub(crate) async fn secrets_csv(
    Extension(user): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    OriginalUri(uri): OriginalUri,
) -> AdminResult<Response> {
    serve("governance-secrets", &uri, &pool, &user).await
}

pub(crate) async fn report_customer_csv(
    Extension(user): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    OriginalUri(uri): OriginalUri,
) -> AdminResult<Response> {
    let id = match param(&uri, "dimension").as_deref() {
        Some("projects") => "report-customer-projects",
        Some("models") => "report-customer-models",
        _ => "report-customer-users",
    };
    serve(id, &uri, &pool, &user).await
}

pub(crate) async fn report_internal_csv(
    Extension(user): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    OriginalUri(uri): OriginalUri,
) -> AdminResult<Response> {
    let id = if param(&uri, "dimension").as_deref() == Some("model") {
        "report-internal-models"
    } else {
        "report-internal-providers"
    };
    serve(id, &uri, &pool, &user).await
}
