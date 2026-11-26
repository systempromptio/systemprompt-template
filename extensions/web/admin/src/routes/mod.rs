//! Router construction for the admin plane.

mod admin;
mod admin_groups;
#[cfg(feature = "mcp")]
mod managed_resources;
#[cfg(feature = "mcp")]
pub(crate) mod managed_state;
mod ssr;
mod ssr_analysis;
mod ssr_bridge;
mod ssr_export;
mod ssr_governance;
mod ssr_platform;

pub(crate) use admin::{
    build_admin_only_routes, build_auth_read_routes, build_self_service_routes,
};
pub use ssr::{AdminSsrShell, admin_ssr_router};
pub use ssr_bridge::bridge_auth_ssr_router;

#[cfg(feature = "mcp")]
fn with_managed_state<S: Clone + Send + Sync + 'static>(
    routes: axum::Router<S>,
    db: &systemprompt::database::DbPool,
    owner: systemprompt::identifiers::UserId,
) -> axum::Router<S> {
    routes.layer(axum::Extension(std::sync::Arc::new(
        managed_state::ManagedState::new(db, owner),
    )))
}

#[cfg(not(feature = "mcp"))]
fn with_managed_state<S>(
    routes: axum::Router<S>,
    _db: &systemprompt::database::DbPool,
    _owner: systemprompt::identifiers::UserId,
) -> axum::Router<S> {
    routes
}
