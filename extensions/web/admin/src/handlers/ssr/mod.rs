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


pub(crate) mod analysis_urls;
mod approvals;
mod context;
#[cfg(feature = "agents")]
pub(crate) mod conversation_header;
mod devices;
pub(crate) mod entity_kind;
#[cfg(any(feature = "gateway", feature = "mcp"))]
pub(crate) mod entity_panel;
pub(crate) mod entity_urls;
pub(crate) mod format;
#[cfg(feature = "gateway")]
mod gateway;
#[cfg(feature = "gateway")]
mod gateway_policies;
pub(crate) mod governance;
#[cfg(feature = "gateway")]
mod governance_quotas;
pub(crate) mod list_view;
mod overview;
pub(crate) mod page;
#[cfg(feature = "gateway")]
pub(crate) mod people_chart;
#[cfg(feature = "gateway")]
pub(crate) mod people_view;
#[cfg(any(feature = "gateway", feature = "mcp"))]
pub(crate) mod pickable_users;
mod roles;
pub(crate) mod secrets_audit;
mod ssr_access_control;
#[cfg(feature = "gateway")]
pub(crate) mod ssr_analytics_dashboard;
#[cfg(feature = "gateway")]
pub(crate) mod ssr_analytics_requests;
mod ssr_bridge_device_link;
mod ssr_bridge_setup;
mod ssr_chain;
#[cfg(feature = "agents")]
mod ssr_context_detail;
#[cfg(feature = "agents")]
mod ssr_conversations_raw;
mod ssr_demo_help;
mod ssr_governance_audit_detail;
#[cfg(feature = "gateway")]
mod ssr_group_detail;
#[cfg(feature = "gateway")]
mod ssr_groups;
pub(crate) mod ssr_helpers;
#[cfg(feature = "agents")]
pub(crate) mod ssr_history;
#[cfg(feature = "agents")]
mod ssr_perf_trace_detail;
#[cfg(feature = "agents")]
pub(crate) mod ssr_perf_traces;
mod ssr_profile;
#[cfg(feature = "mcp")]
mod ssr_projects;
mod ssr_search_resolve;
#[cfg(feature = "agents")]
mod ssr_session_detail;
#[cfg(feature = "agents")]
pub(crate) mod ssr_sessions_list;
mod ssr_settings;
mod ssr_setup;
#[cfg(feature = "agents")]
pub(crate) mod ssr_skills_contexts;
mod ssr_sync;
mod ssr_sync_import;
#[cfg(feature = "agents")]
pub(crate) mod ssr_tools;
mod ssr_users;
pub(crate) mod sync_plane;
#[cfg(feature = "agents")]
pub(crate) mod transcript_view;
pub(crate) mod types;

pub(crate) use approvals::approvals_page;
pub(crate) use devices::devices_page;
#[cfg(feature = "gateway")]
pub(crate) use gateway::{gateway_page, gateway_route_page};
#[cfg(feature = "gateway")]
pub(crate) use gateway_policies::{
    delete_gateway_policy, gateway_policies_page, save_gateway_policy,
};
pub(crate) use governance::governance_page;
#[cfg(feature = "gateway")]
pub(crate) use governance_quotas::governance_quotas_page;
pub(crate) use overview::overview_page;
pub(crate) use roles::roles_page;
pub(crate) use secrets_audit::secrets_audit_page;
pub(crate) use ssr_access_control::access_control_page;
#[cfg(feature = "gateway")]
pub(crate) use ssr_analytics_dashboard::analytics_dashboard_page;
#[cfg(feature = "gateway")]
pub(crate) use ssr_analytics_requests::analytics_requests_page;
pub(crate) use ssr_bridge_device_link::{device_link_approve, device_link_deny, device_link_page};
pub(crate) use ssr_bridge_setup::bridge_setup_page;
pub(crate) use ssr_chain::chain_envelope;
#[cfg(feature = "agents")]
pub(crate) use ssr_context_detail::context_detail_page;
#[cfg(feature = "agents")]
pub(crate) use ssr_conversations_raw::conversations_raw;
pub(crate) use ssr_governance_audit_detail::governance_audit_detail_page;
#[cfg(feature = "gateway")]
pub(crate) use ssr_group_detail::group_detail_page;
#[cfg(feature = "gateway")]
pub(crate) use ssr_groups::groups_page;
pub(crate) use ssr_helpers::{branding_context, render_typed_page};
#[cfg(feature = "agents")]
pub(crate) use ssr_history::{
    conversations_page, history_conversation_page, history_page, history_search,
};
#[cfg(feature = "agents")]
pub(crate) use ssr_perf_trace_detail::perf_trace_detail_page;
#[cfg(feature = "agents")]
pub(crate) use ssr_perf_traces::perf_traces_page;
pub(crate) use ssr_profile::{issue_bridge_code, profile_page};
#[cfg(feature = "mcp")]
pub(crate) use ssr_projects::{project_detail_page, project_report_page, projects_page};
pub(crate) use ssr_search_resolve::search_resolve;
#[cfg(feature = "agents")]
pub(crate) use ssr_session_detail::session_detail_page;
#[cfg(feature = "agents")]
pub(crate) use ssr_sessions_list::sessions_list_page;
pub(crate) use ssr_settings::settings_page;
pub(crate) use ssr_setup::setup_page;
#[cfg(feature = "agents")]
pub(crate) use ssr_skills_contexts::skills_contexts_page;
pub(crate) use ssr_sync::sync_page;
pub(crate) use ssr_sync_import::import_preview_page;
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
}

/// Which sign-in providers the web member contributed to core's `SsoSlot`,
/// so the login page offers exactly the doors core serves.
#[derive(Debug, Clone, Copy, Default)]
pub struct SignInOptions {
    pub adfs: bool,
}

pub(crate) async fn login_page(
    user_ctx: Option<Extension<UserContext>>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Extension(sign_in): Extension<SignInOptions>,
    Query(params): Query<LoginParams>,
) -> AdminHtmlResult<Response> {
    if let Some(Extension(user_ctx)) = user_ctx {
        let target = if user_ctx.is_console {
            "/admin"
        } else {
            "/admin/profile"
        };
        return Ok(Redirect::to(target).into_response());
    }

    let redirect_encoded = params
        .redirect
        .as_deref()
        .and_then(systemprompt_web_shared::redirect::same_origin_path)
        .map(|target| urlencoding::encode(&target).into_owned());

    let ctx = LoginContext {
        shell: branding_context(&engine),
        redirect_encoded,
        sso_enabled: sign_in.adfs,
    };
    let html = engine.render("login", &ctx)?;
    Ok(Html(html).into_response())
}

pub(crate) mod analysis;
mod configuration;
#[cfg(feature = "agents")]
pub(crate) mod ssr_artifacts;
#[cfg(feature = "mcp")]
mod ssr_connect;
#[cfg(feature = "mcp")]
mod ssr_connectors;
#[cfg(feature = "mcp")]
pub(crate) mod ssr_connectors_cards;
mod ssr_lifecycle;
mod ssr_observability;
pub(crate) use configuration::configuration_page;
#[cfg(feature = "agents")]
pub(crate) use ssr_artifacts::{artifact_detail_page, artifact_preview, artifacts_page};
#[cfg(feature = "mcp")]
pub(crate) use ssr_connect::connect_page;
#[cfg(feature = "mcp")]
pub(crate) use ssr_connectors::connectors_page;
pub(crate) use ssr_lifecycle::{lifecycle_archive_download, lifecycle_page};
pub(crate) use ssr_observability::observability_page;
#[cfg(feature = "agents")]
pub(crate) use ssr_tools::tools_page;
