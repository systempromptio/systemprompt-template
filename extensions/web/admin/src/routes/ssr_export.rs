//! Export routes: one surface for every table the console renders — the
//! file in any format and column selection, and the preview the dialog
//! counts from (`export::registry` says which tables exist).

use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use sqlx::PgPool;

use crate::export;

pub(super) fn routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/export/{dataset}", get(export::handler::export_file))
        .route(
            "/export/{dataset}/preview",
            get(export::handler::export_preview),
        )
}
