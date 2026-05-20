pub mod access_control;
pub mod catalog;
pub mod cowork;
pub mod demo_register;
pub mod departments;
pub mod devices;
pub mod entity_access;
pub mod gateway;
pub mod gateway_access;
pub mod gateway_catalog;
pub mod hooks_track;
mod jobs;
pub mod magic_link;
mod plugins;
mod plugins_env;
pub mod public_register;
pub mod resources;
pub mod responses;
pub mod secrets;
pub mod share;
pub mod shared;
pub mod ssr;
mod users;
mod webhook;

pub use webhook::{govern_authz, govern_tool_use, track_statusline_event, track_transcript_event};

pub use gateway::{
    create_gateway_route_handler, delete_gateway_route_handler, get_gateway_handler,
    reorder_gateway_routes_handler, update_gateway_route_handler, update_gateway_settings_handler,
};
pub use jobs::list_jobs_handler;
pub use plugins::list_plugins_handler;
pub use plugins_env::{list_plugin_env_handler, resolve_principal};
pub use resources::{get_agent_handler, list_agents_handler};
pub use users::extract_user_from_cookie;
pub use users::{
    create_user_handler, dashboard_handler, delete_user_handler, list_events_handler,
    list_users_handler, update_user_handler, user_detail_handler, user_usage_handler,
};
