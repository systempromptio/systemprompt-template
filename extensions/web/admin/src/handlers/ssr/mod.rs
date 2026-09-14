//! Server-rendered admin pages.
//!
//! Each module owns one page: it builds a typed template context and renders a
//! `.hbs` template from `storage/files/admin/templates/` at request time.

use crate::error::AdminHtmlResult;
use crate::templates::AdminTemplateEngine;
use crate::types::UserContext;
use axum::Extension;
use axum::extract::Query;
use axum::response::{Html, IntoResponse, Redirect, Response};


mod approvals;
mod context;
pub(crate) mod conversation_header;
pub(crate) mod csv;
mod devices;
pub(crate) mod entity_urls;
pub(crate) mod format;
mod gateway;
mod governance;
pub(crate) mod list_view;
mod overview;
pub(crate) mod people_chart;
pub(crate) mod people_view;
mod roles;
mod secrets_audit;
mod ssr_access_control;
mod ssr_analytics_dashboard;
pub(crate) mod ssr_analytics_requests;
mod ssr_bridge_device_link;
mod ssr_bridge_setup;
mod ssr_chain;
mod ssr_context_detail;
mod ssr_conversations_raw;
mod ssr_demo_help;
mod ssr_governance_audit_detail;
mod ssr_group_detail;
mod ssr_groups;
pub(crate) mod ssr_helpers;
pub(crate) mod ssr_history;
mod ssr_perf_trace_detail;
mod ssr_perf_traces;
mod ssr_profile;
mod ssr_projects;
mod ssr_report_customer;
mod ssr_report_internal;
mod ssr_search_resolve;
mod ssr_session_detail;
mod ssr_sessions_list;
mod ssr_settings;
mod ssr_setup;
mod ssr_skills_contexts;
mod ssr_users;
pub(crate) mod transcript_view;
pub(crate) mod types;

pub(crate) use approvals::approvals_page;
pub(crate) use devices::devices_page;
pub(crate) use gateway::gateway_page;
pub(crate) use governance::{governance_csv, governance_page};
pub(crate) use overview::overview_page;
pub(crate) use roles::roles_page;
pub(crate) use secrets_audit::{secrets_audit_csv, secrets_audit_page};
pub(crate) use ssr_access_control::access_control_page;
pub(crate) use ssr_analytics_dashboard::analytics_dashboard_page;
pub(crate) use ssr_analytics_dashboard::csv::cost_csv;
pub(crate) use ssr_analytics_requests::{analytics_requests_csv, analytics_requests_page};
pub(crate) use ssr_bridge_device_link::{device_link_approve, device_link_deny, device_link_page};
pub(crate) use ssr_bridge_setup::bridge_setup_page;
pub(crate) use ssr_chain::chain_envelope;
pub(crate) use ssr_context_detail::context_detail_page;
pub(crate) use ssr_conversations_raw::conversations_raw;
pub(crate) use ssr_governance_audit_detail::governance_audit_detail_page;
pub(crate) use ssr_group_detail::group_detail_page;
pub(crate) use ssr_groups::groups_page;
pub(crate) use ssr_helpers::{branding_context, render_typed_page};
pub(crate) use ssr_history::{
    conversations_page, history_conversation_page, history_page, history_search,
};
pub(crate) use ssr_perf_trace_detail::perf_trace_detail_page;
pub(crate) use ssr_perf_traces::perf_traces_page;
pub(crate) use ssr_profile::{issue_bridge_code, profile_page};
pub(crate) use ssr_projects::{project_detail_page, projects_page};
pub(crate) use ssr_report_customer::csv::report_customer_csv;
pub(crate) use ssr_report_internal::csv::report_internal_csv;
pub(crate) use ssr_search_resolve::search_resolve;
pub(crate) use ssr_session_detail::session_detail_page;
pub(crate) use ssr_sessions_list::sessions_list_page;
pub(crate) use ssr_settings::settings_page;
pub(crate) use ssr_setup::setup_page;
pub(crate) use ssr_skills_contexts::skills_contexts_page;
pub(crate) use ssr_users::{user_detail_by_id_page, user_detail_page, users_page};

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
    // Why: Whether the page offers "Sign in with Systemprompt SSO". Off when no
    // usable ADFS config is loaded, so the UI never advertises a door that
    // only redirects back with `?sso=unavailable`.
    sso_enabled: bool,
    // Why: tells a developer how to get in when SSO is off locally; the same
    // predicate that mounts the redeem route, so the hint never points at a
    // door that is not there.
    dev_login_enabled: bool,
}

pub(crate) async fn login_page(
    user_ctx: Option<Extension<UserContext>>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Extension(sso_deps): Extension<crate::handlers::adfs_auth::AdfsDeps>,
    Query(params): Query<LoginParams>,
) -> AdminHtmlResult<Response> {
    if let Some(Extension(user_ctx)) = user_ctx {
        let target = if user_ctx.is_console {
            "/admin/"
        } else {
            "/admin/profile"
        };
        return Ok(Redirect::to(target).into_response());
    }

    let redirect_encoded = sanitize_login_redirect(params.redirect.as_deref())
        .map(|target| urlencoding::encode(&target).into_owned());

    let ctx = LoginContext {
        shell: branding_context(&engine),
        redirect_encoded,
        sso_enabled: sso_deps.config.is_usable(),
        dev_login_enabled: crate::handlers::dev_login::dev_login_enabled(),
    };
    let html = engine.render("login", &ctx)?;
    Ok(Html(html).into_response())
}

fn sanitize_login_redirect(raw: Option<&str>) -> Option<String> {
    let raw = raw?.trim();
    (raw.starts_with('/') && !raw.starts_with("//")).then(|| raw.to_owned())
}

pub(crate) mod analysis;
