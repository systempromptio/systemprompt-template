use crate::handlers::extract_user_from_cookie;
use crate::templates::AdminTemplateEngine;
use axum::{
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Extension,
};
use systemprompt_web_shared::html_escape;

pub const ACCESS_DENIED_HTML: &str = "<h1>Access Denied</h1><p>Admin access required.</p>";

mod ssr_access_control;
mod ssr_add_passkey;
mod ssr_agents;
pub mod ssr_analytics_requests;
mod ssr_chain;
mod ssr_context_detail;
mod ssr_conversations_raw;
mod ssr_cowork_device_link;
mod ssr_cowork_setup;
mod ssr_demo_help;
mod ssr_demo_register;
mod ssr_external_agents;
mod ssr_governance;
mod ssr_governance_audit_detail;
mod ssr_governance_hooks;
mod ssr_governance_policy_edit;
pub mod ssr_helpers;
mod ssr_management;
mod ssr_mcp;
mod ssr_perf_trace_detail;
mod ssr_perf_traces;
mod ssr_plugins;
mod ssr_profile;
mod ssr_search_resolve;
mod ssr_session_detail;
mod ssr_settings;
mod ssr_setup;
mod ssr_skills;
mod ssr_skills_contexts;
mod ssr_users;
mod ssr_users_sessions;
pub mod types;

pub use ssr_access_control::access_control_page;
pub use ssr_add_passkey::add_passkey_page;
pub use ssr_agents::{agent_edit_page, agents_page};
pub use ssr_analytics_requests::analytics_requests_page;
pub use ssr_chain::chain_envelope;
pub use ssr_context_detail::context_detail_page;
pub use ssr_conversations_raw::conversations_raw;
pub use ssr_cowork_device_link::{device_link_approve, device_link_deny, device_link_page};
pub use ssr_cowork_setup::cowork_setup_page;
pub use ssr_demo_register::demo_register_page;
pub use ssr_external_agents::external_agents_page;
pub use ssr_governance::governance_page;
pub use ssr_governance_audit_detail::governance_audit_detail_page;
pub use ssr_governance_hooks::governance_hooks_page;
pub use ssr_governance_policy_edit::{governance_policy_edit_page, governance_policy_toggle};
pub use ssr_helpers::branding_context;
pub use ssr_helpers::render_page;
pub use ssr_management::{
    management_department_detail_page, management_departments_page, management_devices_page,
    management_marketplaces_page, management_skills_page,
};
pub use ssr_mcp::{mcp_edit_page, mcp_servers_page};
pub use ssr_perf_trace_detail::perf_trace_detail_page;
pub use ssr_perf_traces::perf_traces_page;
pub use ssr_plugins::plugins_page;
pub use ssr_profile::profile_page;
pub use ssr_search_resolve::search_resolve;
pub use ssr_session_detail::session_detail_page;
pub use ssr_settings::settings_page;
pub use ssr_setup::setup_page;
pub use ssr_skills::skill_edit_page;
pub use ssr_skills_contexts::skills_contexts_page;
pub use ssr_users::{user_detail_page, users_page};
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
        return Redirect::to("/admin/access/users").into_response();
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
