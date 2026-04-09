use crate::admin::handlers::extract_user_from_cookie;
use crate::admin::templates::AdminTemplateEngine;
use crate::utils::html_escape;
use axum::{
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Extension,
};

pub(crate) const ACCESS_DENIED_HTML: &str = "<h1>Access Denied</h1><p>Admin access required.</p>";

pub(crate) mod charts;
mod ssr_access_control;
mod ssr_add_passkey;
mod ssr_agents;
mod ssr_browse_plugins;
pub(crate) mod ssr_control_center;
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
pub(crate) mod ssr_helpers;
mod ssr_hooks;
mod ssr_jobs;
mod ssr_marketplace;
mod ssr_mcp;
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
mod ssr_traces;
mod ssr_users;
pub(crate) mod types;

pub(crate) use ssr_access_control::access_control_page;
pub(crate) use ssr_add_passkey::add_passkey_page;
pub(crate) use ssr_agents::{agent_edit_page, agents_page};
pub(crate) use ssr_browse_plugins::browse_plugins_page;
pub(crate) use ssr_control_center::build_session_groups_with_status;
pub(crate) use ssr_control_center::control_center_page;
pub(crate) use ssr_control_center::handle_analyse_session;
pub(crate) use ssr_control_center::handle_batch_update_session_status;
pub(crate) use ssr_control_center::handle_generate_report;
pub(crate) use ssr_control_center::handle_rate_session;
pub(crate) use ssr_control_center::handle_rate_skill;
pub(crate) use ssr_control_center::handle_update_session_status;
pub(crate) use ssr_dashboard::dashboard_page;
pub(crate) use ssr_dashboard_report::handle_generate_traffic_report;
pub(crate) use ssr_demo_register::demo_register_page;
pub(crate) use ssr_events::events_page;
pub(crate) use ssr_gamification::achievements_page;
pub(crate) use ssr_gamification::leaderboard_page;
pub(crate) use ssr_governance::governance_page;
pub(crate) use ssr_helpers::branding_context;
pub(crate) use ssr_helpers::render_page;
pub(crate) use ssr_hooks::{hook_edit_page, hooks_page};
pub(crate) use ssr_jobs::jobs_page;
pub(crate) use ssr_marketplace::marketplace_versions_page;
pub(crate) use ssr_mcp::{mcp_edit_page, mcp_servers_page};
pub(crate) use ssr_my_activity::my_activity_page;
pub(crate) use ssr_my_agents::{my_agent_edit_page, my_agents_page};
pub(crate) use ssr_my_hooks::my_hooks_page;
pub(crate) use ssr_my_marketplace::my_marketplace_page;
pub(crate) use ssr_my_mcp_servers::my_mcp_servers_page;
pub(crate) use ssr_my_plugin_view::my_plugin_view_page;
pub(crate) use ssr_my_plugins::{my_plugin_edit_page, my_plugins_page};
pub(crate) use ssr_my_secrets::my_secrets_page;
pub(crate) use ssr_my_skills::{my_skill_edit_page, my_skills_page};
pub(crate) use ssr_org_marketplace::org_marketplace_page;
pub(crate) use ssr_plugins::plugins_page;
pub(crate) use ssr_profile::handle_generate_profile_report;
pub(crate) use ssr_profile::profile_page;
pub(crate) use ssr_settings::settings_page;
pub(crate) use ssr_setup::setup_page;
pub(crate) use ssr_skills::{skill_edit_page, skills_page};
pub(crate) use ssr_traces::traces_page;
pub(crate) use ssr_users::{user_detail_page, users_page};

pub(crate) async fn login_page(Extension(engine): Extension<AdminTemplateEngine>) -> Response {
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

pub(crate) async fn verify_pending_page(
    Extension(engine): Extension<AdminTemplateEngine>,
) -> Response {
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

pub(crate) async fn register_page(
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

pub(crate) fn get_services_path() -> Result<std::path::PathBuf, Box<Response>> {
    super::shared::get_services_path()
}
