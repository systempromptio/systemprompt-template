//! Managed authoring is independent of experiment execution and publication.

use crate::handlers::managed_resources as handlers;
use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};

pub(super) fn reads() -> Router {
    Router::new()
        .route("/managed/sources/{id}", get(handlers::source))
        .route("/managed/revisions/{id}", get(handlers::revision))
        .route(
            "/managed/revisions/{baseline}/compare/{candidate}",
            get(handlers::comparison),
        )
        .route("/managed/revisions/{id}/files", get(handlers::files))
        .route("/managed/revisions/{id}/bundle", get(handlers::bundle))
        .route(
            "/managed/resources/{kind}/{key}/resolution",
            get(handlers::resolution),
        )
        .route(
            "/managed/resources/{id}/publications/{generation}/bundle",
            get(handlers::publication_bundle),
        )
        .route(
            "/managed/resources/{id}/publications",
            get(handlers::publication_history),
        )
        .route("/managed/withdrawals", get(handlers::withdrawal_proposals))
        .route("/managed/distributions", get(handlers::distribution_status))
        .route(
            "/managed/installations",
            get(handlers::installation_receipts),
        )
}

pub(super) fn writes() -> Router {
    Router::new()
        .route(
            "/managed/baselines/super-admin",
            post(crate::handlers::evaluation_baseline::capture_super_admin),
        )
        .route("/managed/sources", post(handlers::create_source))
        .route(
            "/managed/sources/{id}/snapshots",
            post(handlers::capture_snapshot),
        )
        .route("/managed/sources/{id}/sync", post(handlers::sync_git))
        .route("/managed/resources", post(handlers::bind_resource))
        .route("/managed/revisions", post(handlers::create_revision))
        .route(
            "/managed/revisions/{id}/candidates",
            post(handlers::create_candidate),
        )
        .route("/managed/publications", post(handlers::publish))
        .route(
            "/managed/reconciliations",
            post(handlers::begin_reconciliation),
        )
        .route(
            "/managed/reconciliations/{id}/conflicts",
            post(handlers::resolve_conflict),
        )
        .route(
            "/managed/reconciliations/{id}/complete",
            post(handlers::complete_reconciliation),
        )
        .route(
            "/managed/withdrawals/{id}",
            post(handlers::decide_withdrawal),
        )
        .route(
            "/managed/distributions/claim",
            post(handlers::claim_distribution),
        )
        .route(
            "/managed/distributions/complete",
            post(handlers::complete_distribution),
        )
        .route(
            "/managed/installations",
            post(handlers::installation_receipt),
        )
        .layer(DefaultBodyLimit::max(32 * 1024 * 1024))
}
