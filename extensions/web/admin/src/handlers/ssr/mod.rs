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


mod context;
pub(crate) mod entity_urls;
pub(crate) mod format;
pub(crate) mod list_view;
mod ssr_access_control;
mod ssr_add_passkey;
pub(crate) mod ssr_analytics_requests;
mod ssr_chain;
mod ssr_context_detail;
mod ssr_conversations_raw;
mod ssr_demo_help;
mod ssr_demo_register;
mod ssr_demo_trace;
mod ssr_evals;
mod ssr_governance;
mod ssr_governance_audit_detail;
mod ssr_governance_decisions;
mod ssr_governance_hooks;
mod ssr_governance_policy_edit;
pub(crate) mod ssr_helpers;
mod ssr_management;
mod ssr_models;
mod ssr_perf_trace_detail;
mod ssr_perf_traces;
mod ssr_profile;
mod ssr_search_resolve;
mod ssr_session_detail;
mod ssr_sessions_list;
mod ssr_settings;
mod ssr_setup;
mod ssr_skills_contexts;
mod ssr_users;
pub(crate) mod types;

pub(crate) use ssr_access_control::access_control_page;
pub(crate) use ssr_add_passkey::add_passkey_page;
pub(crate) use ssr_analytics_requests::analytics_requests_page;
pub(crate) use ssr_chain::chain_envelope;
pub(crate) use ssr_context_detail::context_detail_page;
pub(crate) use ssr_conversations_raw::conversations_raw;
pub(crate) use ssr_demo_register::demo_register_page;
pub(crate) use ssr_demo_trace::demo_trace_page;
pub(crate) use ssr_evals::{
    eval_promote_case_action, eval_run_action, eval_run_detail_page, evals_page,
};
pub(crate) use ssr_governance::governance_page;
pub(crate) use ssr_governance_audit_detail::governance_audit_detail_page;
pub(crate) use ssr_governance_decisions::governance_decisions_page;
pub(crate) use ssr_governance_hooks::governance_hooks_page;
pub(crate) use ssr_governance_policy_edit::{
    governance_policy_edit_page, governance_policy_toggle,
};
pub(crate) use ssr_helpers::{branding_context, render_typed_page};
pub(crate) use ssr_management::{
    management_access_tokens_page, management_department_detail_page, management_departments_page,
};
pub(crate) use ssr_models::models_page;
pub(crate) use ssr_perf_trace_detail::perf_trace_detail_page;
pub(crate) use ssr_perf_traces::perf_traces_page;
pub(crate) use ssr_profile::profile_page;
pub(crate) use ssr_search_resolve::search_resolve;
pub(crate) use ssr_session_detail::session_detail_page;
pub(crate) use ssr_sessions_list::sessions_list_page;
pub(crate) use ssr_settings::settings_page;
pub(crate) use ssr_setup::setup_page;
pub(crate) use ssr_skills_contexts::skills_contexts_page;
pub(crate) use ssr_users::{user_detail_page, users_page};

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

fn render_unauthenticated(
    engine: &AdminTemplateEngine,
    template: &str,
) -> AdminHtmlResult<Response> {
    let html = engine
        .render(template, &branding_context(engine))
        // Why: lint-ok: error-adapt — render errors are Debug-formatted for the html error page
        .map_err(|e| AdminHtmlError::internal(format!("{template} page render failed: {e:?}")))?;
    Ok(Html(html).into_response())
}

pub(crate) fn get_services_path() -> AdminResult<std::path::PathBuf> {
    super::shared::get_services_path()
}
