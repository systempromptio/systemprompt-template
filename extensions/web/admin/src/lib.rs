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
pub mod env;
pub mod error;
pub mod event_hub;
pub(crate) mod export;
pub mod gateway_entitlement;
pub mod gateway_safety;
pub(crate) mod handlers;
pub mod jobs;
mod marketplace_context;
#[cfg(feature = "mcp")]
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
    AdfsDeps, AdfsError, AssertionClaims, FlowState, clear_state_cookie, permissions_for_roles,
    read_state_cookie, state_cookie,
};
#[cfg(feature = "mcp")]
pub use handlers::connector_auth::router as connector_api_router;
pub use handlers::dev_login::{
    DEV_LOGIN_PATH, dev_login_allowed, dev_login_enabled, dev_login_url,
};
pub use routes::{AdminSsrShell, admin_ssr_router, bridge_auth_ssr_router};
pub use services::{connector_oauth, identity_token};
pub use types::{
    CreateUserRequest, MarketplaceContext, UserContext, UserSummary, UserUsageEvent,
    roles_grant_console,
};

pub mod test_support {
    #[cfg(feature = "mcp")]
    pub use crate::handlers::catalog::mcp::status_of;
    #[cfg(feature = "mcp")]
    pub use crate::handlers::catalog::sorting::{
        direction, matches, preserved_search, sort_headers,
    };
    #[cfg(feature = "agents")]
    pub use crate::handlers::hooks_track::ai_context::build_full_context;
    #[cfg(feature = "agents")]
    pub use crate::handlers::hooks_track::ai_summary::build_request_context;
    #[cfg(feature = "agents")]
    pub use crate::handlers::hooks_track::ai_summary_types::{
        SessionAnalysis, session_analysis_schema, validate_analysis,
    };
    #[cfg(feature = "agents")]
    pub use crate::handlers::hooks_track::commits::{
        ParsedCommit, is_commit_command, parse_commit_stdout, response_stdout,
    };
    #[cfg(feature = "agents")]
    pub use crate::handlers::hooks_track::loc::{LocDelta, compute_loc_delta};
    #[cfg(feature = "agents")]
    pub use crate::handlers::hooks_track::session_summary::GeneratedSessionSummary;
    #[cfg(feature = "mcp")]
    pub use crate::handlers::resolve_principal;
    #[cfg(feature = "mcp")]
    pub use crate::handlers::ssr::analysis::marketplace_versions::diff::{
        Change, ManifestDiff, SkillChange, diff as manifest_diff,
    };
    pub use crate::handlers::ssr::analysis::time::{
        parse as parse_window_bound, render as render_window_bound,
    };
    #[cfg(feature = "agents")]
    pub use crate::handlers::ssr::analysis::{SkillRef, parse_skill_key};
    #[cfg(feature = "mcp")]
    pub use crate::handlers::ssr::ssr_connectors_cards::{
        ConnectorCardView, card as connector_card,
    };
    pub use crate::handlers::ssr::ssr_history::command_name as history_command_name;
    pub use crate::handlers::ssr::transcript_view::{
        ConversationView, EmptyReason, ParsedAssistant, SideCallRowView, SideCallsView, StepView,
        ThreadView, ToolChipView, ToolUseMarker, TranscriptMetaView, TranscriptOptions,
        TranscriptRequestIds, TurnView, build_conversation, meta_view, parse_assistant, preview,
        short_id, strip_system_reminders, tidy_lines, transcript_request_ids,
    };
    pub use crate::handlers::ssr::types::table::{SortColumn, SortHeaderView};
    pub use crate::repositories::analysis::inventory_index::MarketplaceAudience;
    pub use crate::services::connector_accounts::{
        Connection, ConnectionSnapshot, get_connections,
    };
    pub use crate::services::connector_readiness::NotReady;
}

pub fn hooks_webhook_router(
    pool: Arc<PgPool>,
    users: Arc<dyn systemprompt::traits::UserProvider>,
) -> Router {
    let routes = Router::new()
        .route("/hooks/govern", post(handlers::govern_tool_use))
        .route("/govern/authz", post(handlers::govern_authz))
        .route("/hooks/statusline", post(handlers::track_statusline_event))
        .route("/hooks/transcript", post(handlers::track_transcript_event));
    #[cfg(feature = "agents")]
    let routes = routes
        .route(
            "/hooks/track",
            post(handlers::hooks_track::handle_hook_track),
        )
        .layer(Extension(None::<Arc<systemprompt::ai::AiService>>));
    routes
        .layer(Extension(event_hub::EventHub::default()))
        .layer(Extension(users))
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
    db: &systemprompt::database::DbPool,
    read_pool: Arc<PgPool>,
    write_pool: &Arc<PgPool>,
    owner: systemprompt::identifiers::UserId,
) -> Router {
    let admin_only = routes::build_admin_only_routes(db, &read_pool, write_pool, owner);
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
