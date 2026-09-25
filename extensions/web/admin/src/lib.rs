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
pub mod gateway_entitlement;
pub mod gateway_safety;
pub(crate) mod handlers;
mod marketplace_context;
pub mod marketplace_filter;
mod middleware;
pub mod numeric;
pub mod repositories;
mod routes;
pub(crate) mod services;
pub mod templates;
pub mod types;
pub mod util;

use std::sync::Arc;

use axum::routing::{get, post};
use axum::{Extension, Router, middleware as axum_middleware};
use sqlx::PgPool;

pub use handlers::adfs_auth::{
    ADFS_ACS_PATH, ADFS_METADATA_PATH, AdfsConfig, AdfsDeps, AdfsError, AssertionClaims, FlowState,
    clear_state_cookie, group_matches_pattern, permissions_for_roles, read_state_cookie,
    state_cookie,
};
pub use handlers::connector_auth::router as connector_api_router;
pub use handlers::dev_login::{
    DEV_LOGIN_PATH, dev_login_allowed, dev_login_enabled, dev_login_url,
};
pub use handlers::salesforce_auth::{SalesforceConfig, SalesforceDeps, SalesforceError};
pub use routes::{admin_ssr_router, bridge_auth_ssr_router};
pub use services::salesforce_orgs::salesforce_orgs_boot_check;
pub use services::{connector_oauth, salesforce_orgs};
pub use types::{
    CreateUserRequest, MarketplaceContext, UserContext, UserSummary, UserUsageEvent,
    roles_grant_console,
};

pub mod test_support {
    pub use crate::handlers::catalog::mcp::status_of;
    pub use crate::handlers::catalog::sorting::{
        SortColumn, direction, matches, preserved_search, sort_headers,
    };
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
    pub use crate::handlers::ssr::ssr_history::command_name as history_command_name;
    pub use crate::handlers::ssr::transcript_view::{
        ConversationView, EmptyReason, ParsedAssistant, SideCallRowView, SideCallsView, StepView,
        ThreadView, ToolChipView, ToolUseMarker, TranscriptMetaView, TranscriptOptions,
        TranscriptRequestIds, TurnView, build_conversation, meta_view, parse_assistant, preview,
        short_id, strip_system_reminders, tidy_lines, transcript_request_ids,
    };
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

// Why: the desktop bridge's identity endpoint. Mounted under `/api/public` and
// authenticated by the caller's own bridge token, not by the admin session
// cookie — the bridge has no cookie jar.
pub fn bridge_identity_router(pool: Arc<PgPool>) -> Router {
    Router::new()
        .route(
            "/bridge/whoami",
            get(handlers::bridge_whoami::bridge_whoami_handler),
        )
        .with_state(pool)
}

// Why: the per-user Salesforce bearer accessor core's external-MCP client GETs
// at tool-call time. Mounted under `/api/public` and deliberately NOT behind
// `require_auth_middleware`: the caller is core carrying the user's own bridge
// token, which has no session cookie, so the handler authenticates it itself.
pub fn salesforce_api_router(deps: SalesforceDeps) -> Router {
    Router::new()
        .route(
            "/salesforce/token",
            get(handlers::salesforce_auth::salesforce_token_handler),
        )
        .layer(Extension(deps))
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

pub fn admin_router(
    read_pool: Arc<PgPool>,
    write_pool: &Arc<PgPool>,
    owner: systemprompt::identifiers::UserId,
) -> Router {
    let admin_only = routes::build_admin_only_routes(&read_pool, write_pool, owner);
    let auth_reads = routes::build_auth_read_routes(&read_pool);
    let self_service = routes::build_self_service_routes(write_pool);

    admin_only
        .merge(auth_reads)
        .merge(self_service)
        .layer(axum_middleware::from_fn(
            middleware::require_auth_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            read_pool,
            middleware::user_context_middleware,
        ))
}
