//! Shared dashboard routes mounted alongside installation-specific pages.

use crate::handlers;
use axum::Router;
use axum::routing::{get, post};
use sqlx::PgPool;
use std::sync::Arc;

pub(super) fn dashboard_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/users/{user_id}",
            get(handlers::ssr::user_detail_by_id_page),
        )
        .route("/groups", get(handlers::ssr::groups_page))
        .route("/groups/{group_id}", get(handlers::ssr::group_detail_page))
        .route("/projects", get(handlers::ssr::projects_page))
        .route(
            "/projects/{project_id}",
            get(handlers::ssr::project_detail_page),
        )
        .route("/roles", get(handlers::ssr::roles_page))
        .route("/devices", get(handlers::ssr::devices_page))
        .route("/devices/pats", post(handlers::devices::issue_pat))
        .route(
            "/devices/pats/{id}",
            axum::routing::delete(handlers::devices::revoke_pat),
        )
        .route(
            "/devices/certs/{id}",
            axum::routing::delete(handlers::devices::revoke_cert),
        )
        .route("/analytics", get(handlers::ssr::analytics_dashboard_page))
        .route("/analytics/cost.csv", get(handlers::ssr::cost_csv))
        .route("/requests.csv", get(handlers::ssr::analytics_requests_csv))
        .route(
            "/governance/warnings.csv",
            get(handlers::ssr::governance_csv),
        )
        .route(
            "/governance/decisions/{decision_id}",
            get(handlers::ssr::governance_audit_detail_page),
        )
        .route("/governance/approvals", get(handlers::ssr::approvals_page))
        .route(
            "/governance/secrets",
            get(handlers::ssr::secrets_audit_page),
        )
        .route(
            "/governance/secrets.csv",
            get(handlers::ssr::secrets_audit_csv),
        )
        .merge(catalog_routes())
        .route("/gateway", get(handlers::ssr::gateway_page))
        .route(
            "/reports/customer.csv",
            get(handlers::ssr::report_customer_csv),
        )
        .route(
            "/reports/internal.csv",
            get(handlers::ssr::report_internal_csv),
        )
        .route("/history", get(handlers::ssr::history_page))
        .route(
            "/history/conversations/{context_id}",
            get(handlers::ssr::history_conversation_page),
        )
        .route("/api/history/search", get(handlers::ssr::history_search))
        .route(
            "/api/profile/bridge-code",
            post(handlers::ssr::issue_bridge_code),
        )
}

fn catalog_routes() -> Router<Arc<PgPool>> {
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
}
