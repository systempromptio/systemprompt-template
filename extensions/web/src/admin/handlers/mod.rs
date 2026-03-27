pub(crate) mod access_control;
pub(crate) mod demo_register;
pub(crate) mod public_register;
pub(crate) mod export_zip;
pub(crate) mod hooks_track;
pub(crate) mod magic_link;
mod jobs;
mod marketplace;
pub(crate) mod marketplace_git;
pub(crate) mod marketplace_json;
pub(crate) mod marketplace_upload;
pub(crate) mod org_marketplaces;
mod plugins;
mod plugins_crud;
mod plugins_env;
mod plugins_import;
mod plugins_skills;
pub(crate) mod resources;
pub(crate) mod responses;
pub(crate) mod secrets;
pub(crate) mod shared;
pub(crate) mod sse;
pub(crate) mod ssr;
pub(crate) mod user_entities;
mod users;

pub(crate) use jobs::list_jobs_handler;
pub(crate) use marketplace::{
    list_marketplace_handler, marketplace_plugin_users_handler, submit_rating_handler,
    update_visibility_handler,
};
pub(crate) use plugins::{
    create_skill_handler, get_skill_handler, list_plugins_handler, list_skills_handler,
};
pub(crate) use plugins_crud::{
    create_plugin_handler, delete_plugin_handler, get_plugin_detail_handler,
    get_skill_file_handler, list_skill_files_handler, sync_skill_files_handler,
    update_plugin_handler, update_skill_file_handler,
};
pub(crate) use plugins_env::{list_plugin_env_handler, update_plugin_env_handler};
pub(crate) use plugins_import::import_plugin_handler;
pub(crate) use plugins_skills::{
    delete_skill_handler, get_plugin_skills_handler, list_all_skills_handler,
    update_plugin_skills_handler,
};
pub(crate) use resources::{
    create_agent_handler, create_user_agent_handler, delete_agent_handler,
    delete_user_agent_handler, get_agent_handler, list_agents_handler, update_agent_handler,
};
pub(crate) use users::extract_user_from_cookie;
pub(crate) use users::{
    create_user_handler, dashboard_handler, delete_user_handler, list_events_handler,
    list_users_handler, update_user_handler, user_detail_handler, user_usage_handler,
};
