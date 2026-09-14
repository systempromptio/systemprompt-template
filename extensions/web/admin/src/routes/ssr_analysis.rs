//! Analysis, source revision and candidate inspection routes.

use crate::handlers;
use axum::Router;
use axum::routing::{get, post};
use sqlx::PgPool;
use std::sync::Arc;

#[expect(
    clippy::too_many_lines,
    reason = "the declarative route inventory is one analysis surface"
)]
pub(super) fn routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/analysis/campaigns",
            get(handlers::ssr::analysis::campaigns::page)
                .post(handlers::ssr::analysis::campaigns::create),
        )
        .route(
            "/analysis/campaigns/{id}/experiments",
            post(handlers::ssr::analysis::campaigns::attach),
        )
        .route(
            "/analysis/campaigns/{id}/launch",
            post(handlers::ssr::analysis::campaigns::launch),
        )
        .route(
            "/analysis/campaigns/{id}/experiments/{experiment}/report",
            get(handlers::ssr::analysis::campaigns::report),
        )
        .route(
            "/analysis/resources",
            get(handlers::ssr::analysis::portfolio::page),
        )
        .route(
            "/analysis/resources.json",
            get(handlers::ssr::analysis::portfolio::json),
        )
        .route(
            "/analysis/evaluations/{id}",
            get(handlers::ssr::analysis::experiments::detail_page),
        )
        .route(
            "/analysis/evaluations/preflight",
            post(handlers::ssr::analysis::experiments::preflight_page),
        )
        .route(
            "/analysis/evaluations/launch",
            post(handlers::ssr::analysis::experiments::launch),
        )
        .route(
            "/analysis/evaluations/approvals",
            post(handlers::ssr::analysis::experiments::decide_approval),
        )
        .route(
            "/analysis/evaluations/{id}/cancel",
            post(handlers::ssr::analysis::experiments::cancel),
        )
        .route(
            "/analysis/revisions/{id}/edit",
            get(handlers::ssr::analysis::candidates::edit_page)
                .post(handlers::ssr::analysis::candidates::save_candidate)
                .layer(axum::extract::DefaultBodyLimit::max(4 * 1024 * 1024)),
        )
        .route(
            "/analysis/revisions/{baseline}/compare/{candidate}",
            get(handlers::ssr::analysis::candidates::comparison_page),
        )
        .route(
            "/analysis/versions",
            get(handlers::ssr::analysis::versions::resources_page),
        )
        .route(
            "/analysis/versions/capture-baseline",
            post(handlers::evaluation_baseline::capture_super_admin_page),
        )
        .route(
            "/analysis/versions/{id}",
            get(handlers::ssr::analysis::versions::resource_page),
        )
        .route(
            "/analysis/revisions/{id}",
            get(handlers::ssr::analysis::versions::revision_page),
        )
        .route("/analysis", get(handlers::ssr::analysis::skills_page))
        .route(
            "/analysis/skills",
            get(handlers::ssr::analysis::skills_page),
        )
        .route(
            "/analysis/evaluations",
            get(handlers::ssr::analysis::experiments::list_page),
        )
        .route(
            "/analysis/impact",
            get(handlers::ssr::analysis::impact::page),
        )
        .route(
            "/analysis/impact/capture",
            post(handlers::ssr::analysis::impact::capture_failure),
        )
        .route(
            "/analysis/publications",
            get(handlers::ssr::analysis::lifecycle::page),
        )
        .route(
            "/analysis/publications/review",
            post(handlers::ssr::analysis::lifecycle::review),
        )
        .route(
            "/analysis/publications/withdrawals",
            post(handlers::ssr::analysis::lifecycle::decide_withdrawal),
        )
}
