pub mod dashboard_grp;
pub mod effective_plugins;
pub mod governance_grp;
pub mod marketplace_grp;
pub mod mcp_grp;
pub mod plugins_grp;
pub mod publishing_grp;
pub mod secrets_grp;
pub mod users_grp;

// Re-export all grouped modules at the top level so existing
// `crate::repositories::<name>` paths continue to resolve unchanged.
pub use dashboard_grp::*;
pub use effective_plugins::{count_org_entity_additions, list_effective_enriched_plugins};
pub use governance_grp::*;
pub use marketplace_grp::*;
pub use mcp_grp::*;
pub use plugins_grp::*;
pub use publishing_grp::*;
pub use secrets_grp::*;
pub use users_grp::*;

pub use agents::{create_agent, delete_agent, find_agent, list_agents, update_agent};
pub use dashboard::{get_dashboard_data, list_event_breakdown, list_events};
pub use export::{generate_export_bundles, ExportParams};
pub use jobs::list_jobs;
pub use marketplace::{
    list_plugin_ratings, list_plugin_usage, list_plugin_users, list_visibility_rules,
    set_visibility_rules, upsert_rating,
};
pub use marketplace_sync_status::mark_user_dirty;
pub use plugin_crud::{
    create_plugin, delete_plugin, find_plugin_detail, import_plugin_bundle, update_plugin,
};
pub use plugin_env::{
    delete_plugin_env_var, list_all_user_env_vars, list_plugin_env_vars, upsert_plugin_env_var,
};
pub use plugin_maps::build_entity_plugin_maps;
pub(crate) use plugin_resolvers::read_skill_required_secrets;
pub use plugins::{
    count_marketplace_items, list_all_skill_ids, list_plugin_skill_ids, list_plugins_for_roles,
    list_plugins_for_roles_full, update_plugin_skills, MarketplaceCounts,
};
pub use skill_files::{
    find_skill_file, list_skill_files, sync_skill_files, update_skill_file_content,
};
pub use skill_secrets::{
    delete_skill_secret, list_all_user_skill_secrets, list_skill_secrets,
    resolve_secrets_for_skill, upsert_skill_secret,
};
pub use user_agents::{
    create_user_agent, delete_user_agent, fetch_agent_plugin_assignments, get_or_create_user_agent,
    list_user_agents, update_user_agent,
};
pub use user_hooks::{
    create_user_hook, delete_user_hook, find_user_hook, get_hook_event_breakdown,
    get_hook_summary_stats, get_hook_timeseries, list_user_hooks, toggle_user_hook,
    update_user_hook,
};
pub use user_plugin_detail::get_plugin_with_associations;
pub use user_plugins::{
    count_user_plugin_items, create_user_plugin, delete_user_plugin, find_plugin_with_associations,
    find_user_plugin, is_entity_in_platform_plugin, list_user_plugins, list_user_plugins_enriched,
    set_plugin_agents, set_plugin_mcp_servers, set_plugin_skills, update_user_plugin,
};
pub use user_queries::fetch_department_stats;
pub use user_skills::{
    create_user_skill, delete_user_skill, fetch_agent_usage_counts, fetch_skill_avg_ratings,
    fetch_skill_usage_counts, find_agent_skill, get_or_create_user_skill, list_agent_skills,
    list_user_skills, update_user_skill,
};
pub use users::{
    create_user, delete_user, delete_user_complete, fetch_distinct_roles, fetch_user_ranks,
    fetch_user_roles, find_user_detail, get_user_event_type_counts, get_user_sessions,
    get_user_top_tools, get_user_usage, list_user_events, list_users, update_user, UserRank,
};
pub use webhook::{insert_plugin_usage_event, UsageEventParams};
