pub mod access_control;
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
mod marketplace;
mod plugins;
mod plugins_crud;
mod plugins_env;
mod plugins_skills;
pub mod public_register;
pub mod resources;
pub mod responses;
pub mod secrets;
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
pub use marketplace::{
    list_marketplace_handler, marketplace_plugin_users_handler, submit_rating_handler,
    update_visibility_handler,
};
pub use plugins::{
    create_skill_handler, get_skill_handler, list_plugins_handler, list_skills_handler,
};
pub use plugins_crud::{
    create_plugin_handler, delete_plugin_handler, get_plugin_detail_handler,
    update_plugin_handler,
};
pub use plugins_env::{list_plugin_env_handler, update_plugin_env_handler};
pub use plugins_skills::{
    delete_skill_handler, get_plugin_skills_handler, list_all_skills_handler,
    update_plugin_skills_handler,
};
pub use resources::{
    create_agent_handler, create_user_agent_handler, delete_agent_handler,
    delete_user_agent_handler, get_agent_handler, list_agents_handler, update_agent_handler,
};
pub use users::extract_user_from_cookie;
pub use users::{
    create_user_handler, dashboard_handler, delete_user_handler, list_events_handler,
    list_users_handler, update_user_handler, user_detail_handler, user_usage_handler,
};
