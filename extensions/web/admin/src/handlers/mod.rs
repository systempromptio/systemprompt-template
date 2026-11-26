//! HTTP handlers for the admin plane.
//!
//! SSR page handlers live in [`ssr`]; everything else here is JSON API or
//! webhook intake. Handlers own status mapping and call repositories for data.

pub(crate) mod access_control;
pub(crate) mod adfs_auth;
pub(crate) mod approvals;
pub(crate) mod bridge_whoami;
#[cfg(feature = "mcp")]
pub(crate) mod catalog;
#[cfg(feature = "mcp")]
pub(crate) mod connector_reprovision;
pub(crate) mod devices;
pub(crate) mod entity_access;
#[cfg(feature = "gateway")]
pub(crate) mod gateway;
pub(crate) mod gateway_access;
#[cfg(feature = "gateway")]
pub(crate) mod gateway_catalog;
pub(crate) mod groups;
#[cfg(feature = "agents")]
pub(crate) mod hooks_track;
mod jobs;
#[cfg(feature = "mcp")]
pub(crate) mod managed_resources;
#[cfg(feature = "mcp")]
mod plugins;
#[cfg(feature = "mcp")]
mod plugins_env;
pub(crate) mod projects;
#[cfg(feature = "agents")]
pub(crate) mod resources;
pub(crate) mod responses;
pub(crate) mod roles;
pub(crate) mod scope_defaults;
pub(crate) mod secrets;
pub(crate) mod self_service;
pub(crate) mod share;
pub(crate) mod shared;
pub(crate) mod ssr;
pub(crate) mod sync;
mod user_sessions;
mod users;
pub(crate) mod users_bootstrap;
pub(crate) mod webhook;

#[cfg(feature = "agents")]
pub(crate) use webhook::track_transcript_event;
pub(crate) use webhook::{govern_authz, govern_tool_use, track_statusline_event};

#[cfg(feature = "gateway")]
pub(crate) use gateway::{
    create_gateway_route_handler, delete_gateway_route_handler, gateway_config_view,
    get_gateway_handler, reorder_gateway_routes_handler, update_gateway_route_handler,
    update_gateway_settings_handler,
};
pub(crate) use jobs::list_jobs_handler;
#[cfg(feature = "mcp")]
pub(crate) use plugins::list_plugins_handler;
#[cfg(feature = "mcp")]
pub(crate) use plugins_env::list_plugin_env_handler;
#[cfg(feature = "mcp")]
pub use plugins_env::resolve_principal;
#[cfg(feature = "agents")]
pub(crate) use resources::{get_agent_handler, list_agents_handler};
pub(crate) use user_sessions::{
    list_user_sessions_handler, revoke_all_user_sessions_handler, revoke_user_session_handler,
};
pub(crate) use users::{
    dashboard_handler, delete_user_handler, extract_token_from_headers, extract_user_from_cookie,
    list_events_handler, list_users_handler, update_user_handler, user_detail_handler,
    user_usage_handler,
};

#[cfg(feature = "mcp")]
pub(crate) mod connector_auth;
