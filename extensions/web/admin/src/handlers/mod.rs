//! HTTP handlers for the admin plane.
//!
//! SSR page handlers live in [`ssr`]; everything else here is JSON API or
//! webhook intake. Handlers own status mapping and call repositories for data.

pub(crate) mod access_control;
pub(crate) mod catalog;
pub(crate) mod departments;
pub(crate) mod devices;
pub(crate) mod entity_access;
pub(crate) mod gateway;
pub(crate) mod gateway_access;
pub(crate) mod gateway_catalog;
pub(crate) mod hooks_track;
pub(crate) mod invite_accept;
pub(crate) mod invites;
mod jobs;
pub(crate) mod organizations;
pub(crate) mod passkey_auth;
mod plugins;
mod plugins_env;
pub(crate) mod resources;
pub(crate) mod responses;
pub(crate) mod salesforce_auth;
pub(crate) mod secrets;
pub(crate) mod share;
pub(crate) mod shared;
pub(crate) mod ssr;
mod user_sessions;
mod users;
pub(crate) mod users_bootstrap;
pub(crate) mod webhook;

pub(crate) use webhook::{
    govern_authz, govern_tool_use, track_statusline_event, track_transcript_event,
};

pub(crate) use gateway::{
    create_gateway_route_handler, delete_gateway_route_handler, get_gateway_handler,
    reorder_gateway_routes_handler, update_gateway_route_handler, update_gateway_settings_handler,
};
pub(crate) use jobs::list_jobs_handler;
pub(crate) use plugins::list_plugins_handler;
pub(crate) use plugins_env::list_plugin_env_handler;
pub use plugins_env::resolve_principal;
pub(crate) use resources::{get_agent_handler, list_agents_handler};
pub(crate) use user_sessions::{
    list_user_sessions_handler, revoke_all_user_sessions_handler, revoke_user_session_handler,
};
pub(crate) use users::{
    dashboard_handler, delete_user_handler, extract_user_from_cookie, list_events_handler,
    list_users_handler, update_user_handler, user_detail_handler, user_usage_handler,
};
