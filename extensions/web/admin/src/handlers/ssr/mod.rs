//! Server-rendered admin pages.
//!
//! Each module owns one page: it builds a typed template context and renders a
//! `.hbs` template from `storage/files/admin/templates/` at request time.

use crate::error::AdminHtmlResult;
use crate::templates::AdminTemplateEngine;
use axum::Extension;
use axum::extract::Query;
use axum::response::{Html, IntoResponse, Response};


mod context;
pub(crate) mod entity_urls;
pub(crate) mod format;
pub(crate) mod list_view;
mod ssr_analytics_dashboard;
pub(crate) mod ssr_analytics_requests;
pub(crate) mod ssr_analytics_user;
mod ssr_bridge_device_link;
mod ssr_bridge_setup;
mod ssr_chain;
mod ssr_context_detail;
mod ssr_conversations_raw;
mod ssr_demo_help;
mod ssr_enterprises;
mod ssr_governance_audit_detail;
pub(crate) mod ssr_helpers;
mod ssr_invite;
mod ssr_management;
mod ssr_perf_trace_detail;
mod ssr_perf_traces;
mod ssr_profile;
mod ssr_report_customer;
mod ssr_report_internal;
mod ssr_search_resolve;
mod ssr_session_detail;
mod ssr_settings;
mod ssr_setup;
mod ssr_skills_contexts;
mod ssr_users;
mod ssr_users_sessions;
pub(crate) mod types;

pub(crate) use ssr_analytics_dashboard::analytics_dashboard_page;
pub(crate) use ssr_analytics_requests::analytics_requests_page;
pub(crate) use ssr_analytics_user::analytics_user_page;
pub(crate) use ssr_bridge_device_link::{device_link_approve, device_link_deny, device_link_page};
pub(crate) use ssr_bridge_setup::bridge_setup_page;
pub(crate) use ssr_chain::chain_envelope;
pub(crate) use ssr_context_detail::context_detail_page;
pub(crate) use ssr_conversations_raw::conversations_raw;
pub(crate) use ssr_enterprises::{enterprise_detail_page, enterprises_page};
pub(crate) use ssr_governance_audit_detail::governance_audit_detail_page;
pub(crate) use ssr_helpers::{branding_context, render_typed_page};
pub(crate) use ssr_invite::invite_accept_page;
pub(crate) use ssr_management::{management_department_detail_page, management_departments_page};
pub(crate) use ssr_perf_trace_detail::perf_trace_detail_page;
pub(crate) use ssr_perf_traces::perf_traces_page;
pub(crate) use ssr_profile::{issue_bridge_code, profile_page};
pub(crate) use ssr_report_customer::report_customer_page;
pub(crate) use ssr_report_internal::report_internal_page;
pub(crate) use ssr_search_resolve::search_resolve;
pub(crate) use ssr_session_detail::session_detail_page;
pub(crate) use ssr_settings::settings_page;
pub(crate) use ssr_setup::setup_page;
pub(crate) use ssr_skills_contexts::skills_contexts_page;
pub(crate) use ssr_users::{user_detail_page, users_page};
pub(crate) use ssr_users_sessions::users_sessions_page;

#[derive(serde::Deserialize)]
pub(crate) struct LoginParams {
    redirect: Option<String>,
}

#[derive(serde::Serialize)]
struct LoginContext<'a> {
    #[serde(flatten)]
    shell: context::BrandingShell<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    redirect_encoded: Option<String>,
    // Why: Whether the page offers the "create an account" path. The template
    // renders the invite-only wording when this is false, so the UI never
    // advertises a door the server refuses.
    self_registration_open: bool,
}

pub(crate) async fn login_page(
    Extension(engine): Extension<AdminTemplateEngine>,
    Extension(sf_deps): Extension<crate::handlers::salesforce_auth::SalesforceDeps>,
    Query(params): Query<LoginParams>,
) -> AdminHtmlResult<Response> {
    let redirect_encoded = sanitize_login_redirect(params.redirect.as_deref())
        .map(|target| urlencoding::encode(&target).into_owned());

    let ctx = LoginContext {
        shell: branding_context(&engine),
        redirect_encoded,
        self_registration_open: sf_deps.config.allow_self_registration,
    };
    let html = engine.render("login", &ctx)?;
    Ok(Html(html).into_response())
}

fn sanitize_login_redirect(raw: Option<&str>) -> Option<String> {
    let raw = raw?.trim();
    (raw.starts_with('/') && !raw.starts_with("//")).then(|| raw.to_owned())
}
