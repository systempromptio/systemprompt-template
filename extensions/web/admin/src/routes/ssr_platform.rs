//! Server-rendered Platform pages: the catalog (MCP servers, marketplaces,
//! plugins, skills), the gateway and its route/policy editors, Configuration,
//! lifecycle and retention, Code sync, and the observability exporter.
//!
//! Split from `ssr.rs` on size alone; the sidebar group and the handlers are
//! unchanged.

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use sqlx::PgPool;

use crate::handlers;

// Why: sidebar group 5 — the installable units declared in `services/*.yaml`,
// flattened out of the old `/catalog/` prefix, plus the gateway that routes
// model traffic to the providers behind them.
pub(super) fn routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/mcp", get(handlers::catalog::mcp::mcp_servers_page))
        .route(
            "/mcp/{mcp_id}",
            get(handlers::catalog::mcp::mcp_detail_page),
        )
        .route(
            "/marketplaces",
            get(handlers::catalog::marketplaces::marketplaces_page),
        )
        .route(
            "/marketplaces/{marketplace_id}",
            get(handlers::catalog::marketplaces::marketplace_detail_page),
        )
        .route("/plugins", get(handlers::catalog::plugins_page))
        .route(
            "/plugins/{plugin_id}",
            get(handlers::catalog::plugin_detail_page),
        )
        .route("/skills", get(handlers::catalog::skills_page))
        .route(
            "/skills/{skill_id}",
            get(handlers::catalog::skill_detail_page),
        )
        .route("/gateway", get(handlers::ssr::gateway_page))
        // Why: the Platform group's home — every kind of configuration the
        // instance loads, where it comes from and whether the database
        // agrees.
        .route("/configuration", get(handlers::ssr::configuration_page))
        // Why: the retention ledger — measurements, archives and the health
        // report the retention_* jobs write — beside Configuration, where the
        // windows and the cleanup job's last run are shown.
        .route("/lifecycle", get(handlers::ssr::lifecycle_page))
        .route(
            "/lifecycle/archive/{tier}/{period}/{file}",
            get(handlers::ssr::lifecycle_archive_download),
        )
        // Why: Code sync — the sources and the archive that move declarations
        // between the repository and this instance, and the access review
        // that settles the access-control plane entity by entity.
        .route("/sync", get(handlers::ssr::sync_page))
        .route(
            "/sync/import/{stage_id}",
            get(handlers::ssr::import_preview_page),
        )
        .route(
            "/gateway/routes/{route_id}",
            get(handlers::ssr::gateway_route_page),
        )
        // Why: the policy editor writes `ai_gateway_policies` directly; core
        // re-reads it per request, so these are live edits, not file edits.
        .route(
            "/gateway/policies",
            get(handlers::ssr::gateway_policies_page).post(handlers::ssr::save_gateway_policy),
        )
        .route(
            "/gateway/policies/delete",
            post(handlers::ssr::delete_gateway_policy),
        )
        // Why: the exporter is core's `otlp_export` job and its config is a
        // profile block; this page shows both and triggers the job out of
        // turn. It lives beside Sync because that is where the declared
        // config is read from.
        .route(
            "/system/observability",
            get(handlers::ssr::observability_page),
        )
        .route(
            "/system/observability/export",
            post(handlers::ssr::observability_export_now),
        )
        .route(
            "/system/observability/test",
            post(handlers::ssr::observability_test_connection),
        )
}
