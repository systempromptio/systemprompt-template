pub mod activity_grp;
pub mod analytics_grp;
pub mod control_center_grp;
pub mod cowork_grp;
pub mod dashboard_grp;
pub mod departments_grp;
pub mod governance_grp;
pub mod external_agents_grp;
pub mod infra_grp;
pub mod marketplace_grp;
pub mod mcp_grp;
pub mod perf_grp;
pub mod plugins_grp;
pub mod secrets_grp;
pub mod tier_grp;
pub mod traces_grp;
pub mod users_grp;

pub use cowork_grp::*;
pub use dashboard_grp::*;
pub use departments_grp::{
    assign_user_to_department, create_department, delete_department, get_department,
    get_department_by_name, list_department_members, list_departments,
    list_user_management_aggregates, update_department, UserManagementAggregate,
};
pub use governance_grp::*;
pub use marketplace_grp::*;
pub use mcp_grp::*;
pub use plugins_grp::*;
pub use secrets_grp::*;
pub use users_grp::*;

pub use agents::{create_agent, delete_agent, find_agent, list_agents, update_agent};
pub use governance_grp::gateway::{
    create_route as create_gateway_route, delete_route as delete_gateway_route, ensure_route_ids,
    find_matching_route, find_matching_route_index, find_route_index_by_id, get_gateway_config,
    reorder_routes as reorder_gateway_routes, update_gateway_settings,
    update_route as update_gateway_route,
};
pub use dashboard::{get_dashboard_data, list_event_breakdown, list_events};
pub use jobs::list_jobs;
pub use marketplace::{
    list_plugin_ratings, list_plugin_usage, list_plugin_users, list_visibility_rules,
    set_visibility_rules, upsert_rating,
};
pub use plugin_crud::{create_plugin, delete_plugin, find_plugin_detail, update_plugin};
pub use plugin_env::{
    delete_plugin_env_var, list_all_user_env_vars, list_plugin_env_vars, upsert_plugin_env_var,
};
pub use plugin_maps::build_entity_plugin_maps;
pub(crate) use plugin_resolvers::read_skill_required_secrets;
pub use plugins::{
    count_marketplace_items, list_all_skill_ids, list_plugin_skill_ids, list_plugins_for_roles,
    list_plugins_for_roles_full, update_plugin_skills, MarketplaceCounts,
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
pub use user_queries::{
    fetch_department_stats, fetch_user_identity_rows, IdentitySort,
    SortDir as IdentitySortDir,
};
pub use user_skills::{
    create_user_skill, delete_user_skill, fetch_agent_usage_counts, fetch_skill_avg_ratings,
    fetch_skill_usage_counts, find_agent_skill, get_or_create_user_skill, list_agent_skills,
    list_user_skills, update_user_skill,
};
pub use users::{
    create_user, delete_user, delete_user_complete, fetch_distinct_roles, fetch_user_ranks,
    fetch_user_roles, find_user_detail, get_user_event_type_counts, get_user_roles_department,
    get_user_sessions, get_user_top_tools, get_user_usage, list_user_events, list_users,
    update_user, UserRank,
};
pub use webhook::{insert_plugin_usage_event, UsageEventParams};
