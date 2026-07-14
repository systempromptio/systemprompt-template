pub mod activity_grp;
pub mod analytics_grp;
pub mod bridge_grp;
pub mod dashboard_grp;
pub mod departments_grp;
pub mod external_agents_grp;
pub mod governance_grp;
pub mod infra_grp;
pub mod mcp_grp;
pub mod perf_grp;
pub mod plugins_grp;
pub mod profile_grp;
pub mod secrets_grp;
pub mod traces_grp;
pub mod users_grp;

pub use bridge_grp::*;
pub use dashboard_grp::*;
pub use departments_grp::{
    UserManagementAggregate, UserMarketplaceOverride, assign_user_to_department, create_department,
    delete_department, get_department, get_department_by_name, list_department_members,
    list_department_top_tools, list_departments, list_user_management_aggregates,
    list_user_marketplace_overrides, update_department,
};
pub use governance_grp::*;
pub use mcp_grp::*;
pub use plugins_grp::*;
pub use secrets_grp::*;
pub use users_grp::*;

pub use agents::{create_agent, delete_agent, find_agent, list_agents, update_agent};
pub use dashboard::{get_dashboard_data, list_event_breakdown, list_events};
pub use governance_grp::gateway::{
    create_route as create_gateway_route, delete_route as delete_gateway_route, ensure_route_ids,
    find_matching_route, find_matching_route_index, find_route_index_by_id, get_gateway_config,
    reorder_routes as reorder_gateway_routes, update_gateway_settings,
    update_route as update_gateway_route,
};
pub use jobs::list_jobs;
pub use plugin_env::{
    PluginEnvVarInput, delete_plugin_env_var, list_all_user_env_vars, list_plugin_env_vars,
    upsert_plugin_env_var,
};
pub use plugin_maps::build_entity_plugin_maps;
pub use plugins::{
    MarketplaceCounts, count_marketplace_items, list_agent_catalog, list_all_skill_ids,
    list_plugin_catalog, list_plugin_skill_ids, list_plugins_for_roles,
    list_plugins_for_roles_full, list_skill_catalog, update_plugin_skills,
};
pub use plugins_grp::hooks::list_configured_hooks;

pub use user_queries::{
    IdentityQuery, IdentitySort, SortDir as IdentitySortDir, fetch_department_stats,
    fetch_user_identity_rows,
};
pub use users::{
    UserRuntimeAggregate, UserRuntimeDetail, create_user, delete_user, delete_user_complete,
    fetch_distinct_roles, fetch_user_roles, find_user_detail, get_user_event_type_counts,
    get_user_roles_department, get_user_runtime_detail, get_user_sessions, get_user_top_tools,
    get_user_usage, list_user_events, list_user_runtime_aggregates, list_users, update_user,
};
pub use webhook::{UsageEventParams, insert_plugin_usage_event};
