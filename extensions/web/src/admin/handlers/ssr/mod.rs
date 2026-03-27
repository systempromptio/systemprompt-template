use crate::admin::handlers::extract_user_from_cookie;
use crate::admin::numeric;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, UserContext};
use crate::utils::html_escape;
use axum::{
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Extension,
};
use serde_json::json;

pub(crate) const ACCESS_DENIED_HTML: &str = "<h1>Access Denied</h1><p>Admin access required.</p>";

pub(crate) mod charts;
mod ssr_add_passkey;
mod ssr_browse_plugins;
pub(crate) mod ssr_control_center;
mod ssr_dashboard;
mod ssr_demo_register;
mod ssr_dashboard_activity;
mod ssr_dashboard_helpers;
mod ssr_dashboard_report;
mod ssr_dashboard_traffic;
mod ssr_dashboard_traffic_pages;
mod ssr_dashboard_types;
mod ssr_events;
mod ssr_gamification;
mod ssr_jobs;
mod ssr_marketplace;
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
mod ssr_profile;
mod ssr_settings;
mod ssr_setup;
mod ssr_users;
pub(crate) mod types;

pub(crate) use ssr_add_passkey::add_passkey_page;
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
pub(crate) use ssr_demo_register::demo_register_page;
pub(crate) use ssr_dashboard_report::handle_generate_traffic_report;
pub(crate) use ssr_events::events_page;
pub(crate) use ssr_gamification::achievements_page;
pub(crate) use ssr_gamification::leaderboard_page;
pub(crate) use ssr_jobs::jobs_page;
pub(crate) use ssr_marketplace::marketplace_versions_page;
pub(crate) use ssr_my_activity::my_activity_page;
pub(crate) use ssr_my_agents::{my_agent_edit_page, my_agents_page};
pub(crate) use ssr_my_hooks::my_hooks_page;
pub(crate) use ssr_my_marketplace::my_marketplace_page;
pub(crate) use ssr_my_mcp_servers::my_mcp_servers_page;
pub(crate) use ssr_my_plugin_view::my_plugin_view_page;
pub(crate) use ssr_my_plugins::{my_plugin_edit_page, my_plugins_page};
pub(crate) use ssr_my_secrets::my_secrets_page;
pub(crate) use ssr_my_skills::{my_skill_edit_page, my_skills_page};
pub(crate) use ssr_profile::handle_generate_profile_report;
pub(crate) use ssr_profile::profile_page;
pub(crate) use ssr_settings::settings_page;
pub(crate) use ssr_setup::setup_page;
pub(crate) use ssr_users::{user_detail_page, users_page};

fn branding_context(engine: &AdminTemplateEngine) -> serde_json::Value {
    match engine.branding() {
        Some(b) => json!({"branding": b}),
        None => json!({}),
    }
}

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

pub(crate) fn render_page(
    engine: &AdminTemplateEngine,
    template: &str,
    data: &serde_json::Value,
    user_ctx: &UserContext,
    mkt_ctx: &MarketplaceContext,
) -> Response {
    let mut merged = data.clone();
    if let Some(obj) = merged.as_object_mut() {
        obj.insert(
            "current_user".to_string(),
            json!({
                "user_id": user_ctx.user_id,
                "username": user_ctx.username,
                "roles": user_ctx.roles,
                "is_admin": user_ctx.is_admin,
            }),
        );
        let git_url = format!(
            "{}/api/public/marketplace/{}.git",
            mkt_ctx.site_url, mkt_ctx.user_id
        );
        let cowork_git_url = format!(
            "{}/api/public/marketplace/{}/cowork.git",
            mkt_ctx.site_url, mkt_ctx.user_id
        );
        let install_cmd = format!("/plugin marketplace add {git_url}");
        let mcp_url = format!(
            "{}/api/v1/mcp/skill-manager/mcp",
            mkt_ctx.site_url
        );
        obj.insert(
            "marketplace".to_string(),
            json!({
                "user_id": mkt_ctx.user_id,
                "site_url": mkt_ctx.site_url,
                "total_plugins": mkt_ctx.total_plugins,
                "total_skills": mkt_ctx.total_skills,
                "agents_count": mkt_ctx.agents_count,
                "mcp_count": mkt_ctx.mcp_count,
                "git_url": git_url,
                "cowork_git_url": cowork_git_url,
                "install_cmd": install_cmd,
                "mcp_url": mcp_url,
                "tier_name": mkt_ctx.tier_name,
                "is_premium": mkt_ctx.is_premium,
                "rank_level": mkt_ctx.rank_level,
                "rank_name": mkt_ctx.rank_name,
                "rank_tier": mkt_ctx.rank_tier,
                "total_xp": mkt_ctx.total_xp,
                "xp_progress_pct": numeric::round_to_i64(mkt_ctx.xp_progress_pct),
                "has_completed_onboarding": mkt_ctx.has_completed_onboarding,
                "current_streak": mkt_ctx.current_streak,
                "longest_streak": mkt_ctx.longest_streak,
                "next_rank_name": mkt_ctx.next_rank_name,
                "xp_to_next_rank": mkt_ctx.xp_to_next_rank,
            }),
        );
        obj.entry("page_stats".to_string())
            .or_insert_with(|| json!([]));
        if let Some(branding) = engine.branding() {
            if let Ok(val) = serde_json::to_value(branding) {
                obj.insert("branding".to_string(), val);
            }
        }
    }
    match engine.render(template, &merged) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!(template, error = ?e, "SSR render failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!(
                    "<h1>Template Error</h1><p>{}</p>",
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
