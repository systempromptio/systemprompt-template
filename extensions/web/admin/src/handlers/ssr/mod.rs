//! Server-rendered admin pages.
//!
//! Each module owns one page: it builds a typed template context and renders a
//! `.hbs` template from `storage/files/admin/templates/` at request time.

use crate::error::{AdminHtmlError, AdminHtmlResult, AdminResult};
use crate::handlers::extract_user_from_cookie;
use crate::templates::AdminTemplateEngine;
use axum::Extension;
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Redirect, Response};

pub(crate) const ACCESS_DENIED_HTML: &str = "<h1>Access Denied</h1><p>Admin access required.</p>";

mod ssr_access_control;
mod ssr_add_passkey;
pub(crate) mod ssr_analytics_requests;
mod ssr_bridge_device_link;
mod ssr_bridge_setup;
mod ssr_chain;
mod ssr_context_detail;
mod ssr_conversations_raw;
mod ssr_demo_help;
mod ssr_demo_register;
mod ssr_governance;
mod ssr_governance_audit_detail;
mod ssr_governance_hooks;
mod ssr_governance_policy_edit;
pub(crate) mod ssr_helpers;
mod ssr_management;
mod ssr_perf_trace_detail;
mod ssr_perf_traces;
mod ssr_profile;
mod ssr_search_resolve;
mod ssr_session_detail;
mod ssr_settings;
mod ssr_setup;
mod ssr_skills_contexts;
mod ssr_users;
mod ssr_users_sessions;
pub(crate) mod types;

pub(crate) use ssr_access_control::access_control_page;
pub(crate) use ssr_add_passkey::add_passkey_page;
pub(crate) use ssr_analytics_requests::analytics_requests_page;
pub(crate) use ssr_bridge_device_link::{device_link_approve, device_link_deny, device_link_page};
pub(crate) use ssr_bridge_setup::bridge_setup_page;
pub(crate) use ssr_chain::chain_envelope;
pub(crate) use ssr_context_detail::context_detail_page;
pub(crate) use ssr_conversations_raw::conversations_raw;
pub(crate) use ssr_demo_register::demo_register_page;
pub(crate) use ssr_governance::governance_page;
pub(crate) use ssr_governance_audit_detail::governance_audit_detail_page;
pub(crate) use ssr_governance_hooks::governance_hooks_page;
pub(crate) use ssr_governance_policy_edit::{
    governance_policy_edit_page, governance_policy_toggle,
};
pub(crate) use ssr_helpers::{branding_context, render_page, render_typed_page};
pub(crate) use ssr_management::{
    management_department_detail_page, management_departments_page, management_devices_page,
};
pub(crate) use ssr_perf_trace_detail::perf_trace_detail_page;
pub(crate) use ssr_perf_traces::perf_traces_page;
pub(crate) use ssr_profile::profile_page;
pub(crate) use ssr_search_resolve::search_resolve;
pub(crate) use ssr_session_detail::session_detail_page;
pub(crate) use ssr_settings::settings_page;
pub(crate) use ssr_setup::setup_page;
pub(crate) use ssr_skills_contexts::skills_contexts_page;
pub(crate) use ssr_users::{user_detail_page, users_page};
pub(crate) use ssr_users_sessions::users_sessions_page;

pub(crate) async fn login_page(
    Extension(engine): Extension<AdminTemplateEngine>,
) -> AdminHtmlResult<Response> {
    render_unauthenticated(&engine, "login")
}

pub(crate) async fn verify_pending_page(
    Extension(engine): Extension<AdminTemplateEngine>,
) -> AdminHtmlResult<Response> {
    render_unauthenticated(&engine, "verify-pending")
}

pub(crate) async fn register_page(
    headers: HeaderMap,
    Extension(engine): Extension<AdminTemplateEngine>,
) -> AdminHtmlResult<Response> {
    if extract_user_from_cookie(&headers).is_ok() {
        return Ok(Redirect::to("/admin/access/users").into_response());
    }
    render_unauthenticated(&engine, "register")
}

/// The pages reachable before sign-in, which therefore have no user or
/// marketplace context to inject and cannot go through `render_page`.
fn render_unauthenticated(
    engine: &AdminTemplateEngine,
    template: &str,
) -> AdminHtmlResult<Response> {
    let html = engine
        .render(template, &branding_context(engine))
        .map_err(|e| AdminHtmlError::internal(format!("{template} page render failed: {e:?}")))?;
    Ok(Html(html).into_response())
}

pub(crate) fn get_services_path() -> AdminResult<std::path::PathBuf> {
    super::shared::get_services_path()
}
