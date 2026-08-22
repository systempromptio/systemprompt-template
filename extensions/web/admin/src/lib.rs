//! Admin extension for the Enterprise Demo template.
//!
//! Wires the admin dashboard, governance webhooks, bridge plane, and
//! supporting services onto a shared `PgPool`. Public surface is grouped by
//! concern:
//!
//! - [`admin_router`] — the SSR dashboard (auth-gated; admin-only and
//!   authenticated-read routes are layered together).
//! - [`hooks_webhook_router`] — the four governance webhooks called by gateway
//!   / MCP / Claude Code (`/hooks/track`, `/hooks/govern`, `/govern/authz`,
//!   statusline/transcript ingest).
//! - [`secrets_router`], [`share_manifest_router`] — per-plugin secret
//!   resolution and public manifest sharing.
//!
//! [`repositories`] owns every `sqlx` call; handlers/services never touch
//! the DB directly. Errors normalise on `error::MarketplaceError` via the
//! `MarketplaceError` re-export in [`systemprompt_web_shared`].

pub mod activity;
pub mod assets;
pub mod audit_event_bus;
pub mod authz;
pub mod error;
pub mod event_hub;
pub mod gateway_org_budget;
pub mod gateway_safety;
pub(crate) mod handlers;
mod marketplace_context;
pub mod marketplace_filter;
mod middleware;
pub mod numeric;
pub mod repositories;
mod routes;
pub(crate) mod services;
pub mod slack_alerts;
pub mod templates;
pub mod types;
pub mod util;

use std::sync::Arc;

use axum::routing::{get, post};
use axum::{Extension, Router, middleware as axum_middleware};
use sqlx::PgPool;

pub use handlers::salesforce_auth::{SalesforceConfig, SalesforceDeps, SalesforceError};
pub use routes::{admin_ssr_router, bridge_auth_ssr_router};
pub use services::salesforce_org;
pub use types::{CreateUserRequest, MarketplaceContext, UserContext, UserSummary, UserUsageEvent};

pub mod test_support {
    pub use crate::handlers::hooks_track::ai_context::build_full_context;
    pub use crate::handlers::hooks_track::ai_summary::build_request_context;
    pub use crate::handlers::hooks_track::ai_summary_types::{
        SessionAnalysis, session_analysis_schema, validate_analysis,
    };
    pub use crate::handlers::hooks_track::commits::{
        ParsedCommit, is_commit_command, parse_commit_stdout, response_stdout,
    };
    pub use crate::handlers::hooks_track::loc::{LocDelta, compute_loc_delta};
    pub use crate::handlers::hooks_track::session_summary::GeneratedSessionSummary;
    pub use crate::handlers::resolve_principal;
    pub use crate::handlers::salesforce_auth::select_sf_username;
}

pub fn hooks_webhook_router(
    pool: Arc<PgPool>,
    session_service: Arc<systemprompt::oauth::SessionCreationService>,
) -> Router {
    Router::new()
        .route(
            "/hooks/track",
            post(handlers::hooks_track::handle_hook_track),
        )
        .route("/hooks/govern", post(handlers::govern_tool_use))
        .route("/govern/authz", post(handlers::govern_authz))
        .route("/hooks/statusline", post(handlers::track_statusline_event))
        .route("/hooks/transcript", post(handlers::track_transcript_event))
        .layer(Extension(event_hub::EventHub::default()))
        .layer(Extension(None::<Arc<systemprompt::ai::AiService>>))
        .layer(Extension(session_service))
        .with_state(pool)
}

pub fn share_manifest_router(pool: Arc<PgPool>) -> Router {
    Router::new()
        .route(
            "/share/manifest/{token}",
            get(handlers::share::public_manifest_handler),
        )
        .with_state(pool)
}

pub fn secrets_router(pool: Arc<PgPool>) -> Router {
    Router::new()
        .route(
            "/api/v1/secrets/{plugin_id}/token",
            post(handlers::secrets::create_resolution_token_handler),
        )
        .route(
            "/api/v1/secrets/{plugin_id}/resolve",
            get(handlers::secrets::resolve_secrets_handler),
        )
        .route(
            "/admin/api/secrets/{plugin_id}/audit",
            get(handlers::secrets::audit_log_handler),
        )
        .route(
            "/admin/api/secrets/{plugin_id}/rotate",
            post(handlers::secrets::rotate_handler),
        )
        .with_state(pool)
}

// Why: Core's external-MCP client calls it to obtain a fresh per-user
// Salesforce bearer + instance URL. Self-authenticates via cookie/bearer in the
// handler.
pub fn salesforce_api_router(sf_deps: SalesforceDeps) -> Router {
    Router::new()
        .route(
            "/salesforce/token",
            get(handlers::salesforce_auth::salesforce_token_handler),
        )
        .layer(Extension(sf_deps))
}

pub fn admin_router(read_pool: Arc<PgPool>, write_pool: &Arc<PgPool>) -> Router {
    let admin_only = routes::build_admin_only_routes(&read_pool, write_pool);
    let auth_reads = routes::build_auth_read_routes(&read_pool);

    admin_only
        .merge(auth_reads)
        .layer(axum_middleware::from_fn(
            middleware::require_auth_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            read_pool,
            middleware::user_context_middleware,
        ))
}
