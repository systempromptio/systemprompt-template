pub mod activity;
pub mod autofork;
pub mod event_hub;
pub mod gamification;
pub(crate) mod handlers;
mod middleware;
pub mod numeric;
pub mod repositories;
mod routes;
pub mod slack_alerts;
pub mod templates;
pub mod tier_enforcement;
pub mod tier_limits;
pub mod types;

use std::sync::Arc;

use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Extension, Router,
};
use sqlx::PgPool;

pub use routes::{admin_ssr_router, cowork_auth_ssr_router, workspace_ssr_router};
pub use types::{CreateUserRequest, MarketplaceContext, UsageEvent, UserContext, UserSummary};

pub mod test_support {
    pub use crate::handlers::cowork::plugin_file::{legacy_gone, resolve_within};
}

pub fn hooks_webhook_router(pool: Arc<PgPool>) -> Router {
    Router::new()
        .route(
            "/hooks/track",
            post(handlers::hooks_track::handle_hook_track),
        )
        .route("/hooks/govern", post(handlers::govern_tool_use))
        .route("/hooks/statusline", post(handlers::track_statusline_event))
        .route("/hooks/transcript", post(handlers::track_transcript_event))
        .layer(Extension(event_hub::EventHub::default()))
        .layer(Extension(None::<Arc<systemprompt::ai::AiService>>))
        .layer(Extension(tier_enforcement::TierEnforcementCache::default()))
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

pub fn cowork_router(pool: Arc<PgPool>) -> Router {
    Router::new()
        .route("/v1/cowork/manifest", get(handlers::cowork::manifest::handle))
        .route("/v1/cowork/whoami", get(handlers::cowork::whoami::handle))
        .route(
            "/v1/cowork/plugins/{plugin_id}/{*path}",
            get(handlers::cowork::plugin_file::handle),
        )
        .route(
            "/plugins/{plugin_id}/{*path}",
            get(handlers::cowork::plugin_file::legacy_gone),
        )
        .with_state(pool)
}

pub fn marketplace_git_router(pool: Arc<PgPool>) -> Router {
    Router::new()
        .route(
            "/marketplace/{user_id}",
            get(handlers::marketplace_json::marketplace_json_handler)
                .post(handlers::marketplace_upload::marketplace_upload_handler),
        )
        .route(
            "/marketplace/{user_id}/versions",
            get(handlers::marketplace_upload::marketplace_versions_handler),
        )
        .route(
            "/marketplace/{user_id}/versions/{version_id}",
            get(handlers::marketplace_upload::marketplace_version_detail_handler),
        )
        .route(
            "/marketplace/{user_id}/changelog",
            get(handlers::marketplace_upload::marketplace_changelog_handler),
        )
        .route(
            "/marketplace/{user_id}/restore/{version_id}",
            post(handlers::marketplace_upload::marketplace_restore_handler),
        )
        .route(
            "/marketplace/{user_id}/export/marketplace.zip",
            get(handlers::export_zip::export_marketplace_zip_handler),
        )
        .route(
            "/marketplace/{user_id}/export/cowork.zip",
            get(handlers::export_zip::export_cowork_zip_handler),
        )
        .route(
            "/marketplace/{user_id}/export/plugin/{plugin_id}",
            get(handlers::export_zip::export_plugin_zip_handler),
        )
        .route(
            "/marketplace/{user_id}/cowork/{*path}",
            get(handlers::marketplace_git::cowork_git_handler)
                .post(handlers::marketplace_git::cowork_upload_pack_handler),
        )
        .route(
            "/marketplace/{user_id}/cowork.git/{*path}",
            get(handlers::marketplace_git::cowork_git_handler)
                .post(handlers::marketplace_git::cowork_upload_pack_handler),
        )
        .route(
            "/marketplace/{user_id}/{*path}",
            get(handlers::marketplace_git::marketplace_git_handler)
                .post(handlers::marketplace_git::git_upload_pack_handler),
        )
        .with_state(pool)
}

pub fn admin_router(
    read_pool: Arc<PgPool>,
    write_pool: Arc<PgPool>,
    tier_cache: tier_enforcement::TierEnforcementCache,
) -> Router {
    let admin_only = routes::build_admin_only_routes(&read_pool, &write_pool);
    let auth_reads = routes::build_auth_read_routes(&read_pool);
    let auth_writes = routes::build_auth_write_routes(write_pool);

    admin_only
        .merge(auth_reads)
        .merge(auth_writes)
        .layer(Extension(tier_cache))
        .layer(axum_middleware::from_fn(
            middleware::require_auth_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            read_pool,
            middleware::user_context_middleware,
        ))
}
