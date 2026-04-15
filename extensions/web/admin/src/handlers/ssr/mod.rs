use crate::handlers::extract_user_from_cookie;
use crate::templates::AdminTemplateEngine;
use axum::{
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Extension,
};
use systemprompt_web_shared::html_escape;

pub const ACCESS_DENIED_HTML: &str = "<h1>Access Denied</h1><p>Admin access required.</p>";

pub mod charts;
mod ssr_access_control;
mod ssr_add_passkey;
mod ssr_agent_config;
mod ssr_agent_messages;
mod ssr_agent_traces;
mod ssr_agents;
mod ssr_analytics_agents;
mod ssr_analytics_content;
mod ssr_analytics_conversations;
mod ssr_analytics_costs;
mod ssr_analytics_overview;
mod ssr_analytics_requests;
mod ssr_analytics_sessions;
mod ssr_analytics_tools;
mod ssr_browse_plugins;
pub mod ssr_control_center;
mod ssr_dashboard;
mod ssr_dashboard_activity;
mod ssr_dashboard_helpers;
mod ssr_dashboard_report;
mod ssr_dashboard_traffic;
mod ssr_dashboard_traffic_pages;
mod ssr_dashboard_types;
mod ssr_demo_help;
mod ssr_demo_register;
mod ssr_events;
mod ssr_gamification;
mod ssr_governance;
mod ssr_governance_audit;
mod ssr_governance_decisions;
mod ssr_governance_hooks;
mod ssr_governance_rate_limits;
mod ssr_governance_rules;
mod ssr_governance_violations;
pub mod ssr_helpers;
mod ssr_hooks;
mod ssr_infra_config;
mod ssr_infra_database;
mod ssr_infra_logs;
mod ssr_infra_services;
mod ssr_jobs;
mod ssr_marketplace;
mod ssr_mcp;
mod ssr_mcp_access;
mod ssr_mcp_tools;
mod ssr_perf_benchmarks;
mod ssr_perf_traces;
mod ssr_my_activity;
mod ssr_my_agents;
mod ssr_my_hooks;
mod ssr_my_marketplace;
mod ssr_my_mcp_servers;
mod ssr_my_plugin_view;
mod ssr_my_plugins;
mod ssr_my_plugins_helpers;
mod ssr_my_secrets;
mod ssr_my_skills;
mod ssr_org_marketplace;
mod ssr_plugins;
mod ssr_profile;
mod ssr_settings;
mod ssr_setup;
mod ssr_skills;
mod ssr_skills_content;
mod ssr_skills_contexts;
mod ssr_skills_files;
mod ssr_skills_plugins;
mod ssr_traces;
mod ssr_users;
mod ssr_users_ip_bans;
mod ssr_users_sessions;
pub mod types;

pub use ssr_access_control::access_control_page;
pub use ssr_add_passkey::add_passkey_page;
pub use ssr_agent_config::agent_config_page;
pub use ssr_agent_messages::agent_messages_page;
pub use ssr_agent_traces::agent_traces_page;
pub use ssr_agents::{agent_edit_page, agents_page};
pub use ssr_analytics_agents::analytics_agents_page;
pub use ssr_analytics_content::analytics_content_page;
pub use ssr_analytics_conversations::analytics_conversations_page;
pub use ssr_analytics_costs::analytics_costs_page;
pub use ssr_analytics_overview::analytics_overview_page;
pub use ssr_analytics_requests::analytics_requests_page;
pub use ssr_analytics_sessions::analytics_sessions_page;
pub use ssr_analytics_tools::analytics_tools_page;
pub use ssr_browse_plugins::browse_plugins_page;
pub use ssr_control_center::build_session_groups_with_status;
pub use ssr_control_center::control_center_page;
pub use ssr_control_center::handle_analyse_session;
pub use ssr_control_center::handle_batch_update_session_status;
pub use ssr_control_center::handle_generate_report;
pub use ssr_control_center::handle_rate_session;
pub use ssr_control_center::handle_rate_skill;
pub use ssr_control_center::handle_update_session_status;
pub use ssr_dashboard::dashboard_page;
pub use ssr_dashboard_report::handle_generate_traffic_report;
pub use ssr_demo_register::demo_register_page;
pub use ssr_events::events_page;
pub use ssr_gamification::achievements_page;
pub use ssr_gamification::leaderboard_page;
pub use ssr_governance::governance_page;
pub use ssr_governance_audit::governance_audit_page;
pub use ssr_governance_decisions::governance_decisions_page;
pub use ssr_governance_hooks::governance_hooks_page;
pub use ssr_governance_rate_limits::governance_rate_limits_page;
pub use ssr_governance_rules::governance_rules_page;
pub use ssr_governance_violations::governance_violations_page;
pub use ssr_helpers::branding_context;
pub use ssr_helpers::{render_page, render_typed_page};
pub use ssr_hooks::{hook_edit_page, hooks_page};
pub use ssr_infra_config::infra_config_page;
pub use ssr_infra_database::infra_database_page;
pub use ssr_infra_logs::infra_logs_page;
pub use ssr_infra_services::infra_services_page;
pub use ssr_jobs::jobs_page;
pub use ssr_marketplace::marketplace_versions_page;
pub use ssr_mcp::{mcp_edit_page, mcp_servers_page};
pub use ssr_mcp_access::mcp_access_page;
pub use ssr_mcp_tools::mcp_tools_page;
pub use ssr_perf_benchmarks::perf_benchmarks_page;
pub use ssr_perf_traces::perf_traces_page;
pub use ssr_my_activity::my_activity_page;
pub use ssr_my_agents::{my_agent_edit_page, my_agents_page};
pub use ssr_my_hooks::my_hooks_page;
pub use ssr_my_marketplace::my_marketplace_page;
pub use ssr_my_mcp_servers::my_mcp_servers_page;
pub use ssr_my_plugin_view::my_plugin_view_page;
pub use ssr_my_plugins::{my_plugin_edit_page, my_plugins_page};
pub use ssr_my_secrets::my_secrets_page;
pub use ssr_my_skills::{my_skill_edit_page, my_skills_page};
pub use ssr_org_marketplace::org_marketplace_page;
pub use ssr_plugins::plugins_page;
pub use ssr_profile::handle_generate_profile_report;
pub use ssr_profile::profile_page;
pub use ssr_settings::settings_page;
pub use ssr_setup::setup_page;
pub use ssr_skills::{skill_edit_page, skills_page};
pub use ssr_skills_content::skills_content_page;
pub use ssr_skills_contexts::skills_contexts_page;
pub use ssr_skills_files::skills_files_page;
pub use ssr_skills_plugins::skills_plugins_page;
pub use ssr_traces::traces_page;
pub use ssr_users::{user_detail_page, users_page};
pub use ssr_users_ip_bans::users_ip_bans_page;
pub use ssr_users_sessions::users_sessions_page;
pub async fn login_page(Extension(engine): Extension<AdminTemplateEngine>) -> Response {
    match engine.render("login", &branding_context(&engine)) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!(error = ?e, "Login page render failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!(
                    "<h1>Error</h1><p>{}</p>",
                    html_escape(&e.to_string())
                )),
            )
                .into_response()
        }
    }
}

pub async fn verify_pending_page(Extension(engine): Extension<AdminTemplateEngine>) -> Response {
    match engine.render("verify-pending", &branding_context(&engine)) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!(error = ?e, "Verify-pending page render failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!(
                    "<h1>Error</h1><p>{}</p>",
                    html_escape(&e.to_string())
                )),
            )
                .into_response()
        }
    }
}

pub async fn register_page(
    headers: HeaderMap,
    Extension(engine): Extension<AdminTemplateEngine>,
) -> Response {
    if extract_user_from_cookie(&headers).is_ok() {
        return Redirect::to("/control-center").into_response();
    }
    match engine.render("register", &branding_context(&engine)) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!(error = ?e, "Register page render failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!(
                    "<h1>Error</h1><p>{}</p>",
                    html_escape(&e.to_string())
                )),
            )
                .into_response()
        }
    }
}

pub fn get_services_path() -> Result<std::path::PathBuf, Box<Response>> {
    super::shared::get_services_path()
}
