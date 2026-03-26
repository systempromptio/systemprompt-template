pub mod access_control;
pub mod agents;
pub mod dashboard;
mod dashboard_aggregates;
mod dashboard_queries;
mod dashboard_queries_extra;
pub mod export;
pub(crate) mod export_auth;
mod export_builders;
mod export_builders_env;
mod export_resolvers;
mod export_scripts;
mod export_scripts_marketplace;
mod export_scripts_templates;
mod export_user;
mod export_validation;
pub mod export_zip;
pub mod github_sync;
pub mod hook_catalog;
pub mod hooks;
pub mod jobs;
pub mod magic_links;
pub mod marketplace;
pub mod marketplace_git;
pub mod marketplace_sync;
mod marketplace_sync_archive;
mod marketplace_sync_parse;
pub mod marketplace_sync_status;
pub mod marketplace_versions;
pub mod mcp_servers;
mod mcp_servers_yaml;
pub mod org_marketplaces;
pub mod plugin_crud;
mod plugin_crud_ops;
pub mod plugin_env;
mod plugin_hook_resolvers;
mod plugin_import;
mod plugin_maps;
mod plugin_resolvers;
pub mod plugins;
pub mod secret_audit;
pub mod secret_crypto;
pub mod secret_keys;
pub mod secret_resolve;
pub mod skill_files;
pub mod skill_secrets;
pub mod usage_aggregations;
pub mod user_agents;
pub mod user_hooks;
pub mod user_mcp_servers;
pub mod user_plugin_associations;
mod user_plugin_detail;
pub mod user_plugin_selections;
pub mod user_plugins;
mod user_queries;
pub mod user_skills;
pub mod users;
pub mod webhook;

pub use agents::{create_agent, delete_agent, get_agent, list_agents, update_agent};
pub use dashboard::{get_dashboard_data, list_events};
pub use dashboard_aggregates::fetch_event_breakdown;
pub use export::generate_export_bundles;
pub use jobs::list_jobs;
pub use marketplace::{
    get_all_plugin_ratings, get_all_plugin_usage, get_all_visibility_rules, get_plugin_users,
    set_visibility_rules, upsert_rating,
};
pub use marketplace_sync_status::mark_user_dirty;
pub use mcp_servers::{
    create_mcp_server, delete_mcp_server, get_mcp_server, list_mcp_servers, update_mcp_server,
};
pub use mcp_servers_yaml::{get_mcp_server_raw_yaml, update_mcp_server_raw_yaml};
pub use plugin_crud::{
    create_plugin, delete_plugin, get_plugin_detail, import_plugin_bundle, update_plugin,
};
pub use plugin_env::{
    delete_plugin_env_var, list_all_user_env_vars, list_plugin_env_vars, upsert_plugin_env_var,
};
pub use plugin_maps::build_entity_plugin_maps;
pub use plugins::{
    count_marketplace_items, get_plugin_skill_ids, list_all_skill_ids, list_plugins_for_roles,
    list_plugins_for_roles_full, load_plugin_onboarding_configs, update_plugin_skills,
    MarketplaceCounts,
};
pub use skill_files::{
    get_skill_file, list_skill_files, sync_skill_files, update_skill_file_content,
};
pub use skill_secrets::{
    delete_skill_secret, list_all_user_skill_secrets, list_skill_secrets,
    resolve_secrets_for_skill, upsert_skill_secret,
};
pub use user_agents::{create_user_agent, delete_user_agent, list_user_agents, update_user_agent};
pub use user_hooks::{
    create_user_hook, delete_user_hook, get_hook_overrides_enabled_map, list_user_hooks,
    update_user_hook, upsert_hook_override_enabled,
};
pub use user_mcp_servers::{
    create_user_mcp_server, delete_user_mcp_server, list_user_mcp_servers, update_user_mcp_server,
};
pub use user_plugin_associations::{
    list_user_plugins_enriched, set_plugin_agents, set_plugin_hooks, set_plugin_mcp_servers,
    set_plugin_skills,
};
pub use user_plugin_detail::get_plugin_with_associations;
pub use user_plugins::{
    create_user_plugin, delete_user_plugin, get_user_plugin, list_user_plugins, update_user_plugin,
};
pub use user_queries::{
    fetch_department_stats, fetch_distinct_roles, get_user_detail, get_user_usage,
};
pub use user_skills::{
    create_user_skill, delete_user_skill, fetch_agent_usage_counts, fetch_skill_avg_ratings,
    fetch_skill_usage_counts, get_agent_skill, get_agent_skills_enabled_map, list_agent_skills,
    list_user_skills, update_agent_skill_enabled, update_user_skill,
};
pub use users::{create_user, delete_user, list_users, update_user};
pub use webhook::{
    extract_transcript_tokens, get_session_entries_counted, insert_plugin_usage_event,
    insert_session_transcript, update_transcript_tokens, upsert_plugin_installation,
};
