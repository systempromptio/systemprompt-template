pub(crate) mod access_control;
pub(crate) mod bridge;
pub(crate) mod catalog;
pub(crate) mod demo_register;
pub(crate) mod departments;
pub(crate) mod devices;
pub(crate) mod entity_access;
pub(crate) mod gateway;
pub(crate) mod gateway_access;
pub(crate) mod gateway_catalog;
pub(crate) mod hooks_track;
mod jobs;
pub(crate) mod magic_link;
mod plugins;
mod plugins_env;
pub(crate) mod public_register;
pub(crate) mod resources;
pub(crate) mod responses;
pub(crate) mod secrets;
pub(crate) mod share;
pub(crate) mod shared;
pub(crate) mod ssr;
mod users;
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
pub(crate) use users::{
    create_user_handler, dashboard_handler, delete_user_handler, extract_user_from_cookie,
    list_events_handler, list_users_handler, update_user_handler, user_detail_handler,
    user_usage_handler,
};
