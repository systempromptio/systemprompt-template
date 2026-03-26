use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, UserContext};
use crate::utils::html_escape;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension,
};
use serde_json::json;

pub(crate) const ACCESS_DENIED_HTML: &str = "<h1>Access Denied</h1><p>Admin access required.</p>";

mod chart_helpers;
mod ssr_access_control;
mod ssr_add_passkey;
mod ssr_agents;
mod ssr_browse_plugins;
mod ssr_dashboard;
mod ssr_events;
mod ssr_gamification;
mod ssr_hooks;
mod ssr_jobs;
mod ssr_marketplace;
mod ssr_mcp;
mod ssr_my_agents;
mod ssr_my_hooks;
mod ssr_my_marketplace;
mod ssr_my_mcp;
mod ssr_my_plugin_view;
mod ssr_my_plugins;
mod ssr_my_secrets;
mod ssr_my_skills;
mod ssr_org_marketplace_edit;
mod ssr_org_marketplaces;
mod ssr_plugin_edit;
mod ssr_plugins;
mod ssr_skills;
mod ssr_users;

pub(crate) use chart_helpers::{compute_area_chart_data, compute_bar_chart, compute_hourly_chart};
pub(crate) use ssr_access_control::access_control_page;
pub(crate) use ssr_add_passkey::add_passkey_page;
pub(crate) use ssr_agents::{agent_edit_page, agents_page};
pub(crate) use ssr_browse_plugins::browse_plugins_page;
pub(crate) use ssr_dashboard::dashboard_page;
pub(crate) use ssr_events::events_page;
pub(crate) use ssr_gamification::{achievements_page, leaderboard_page};
pub(crate) use ssr_hooks::{hook_edit_page, hooks_page};
pub(crate) use ssr_jobs::jobs_page;
pub(crate) use ssr_marketplace::{marketplace_page, marketplace_versions_page};
pub(crate) use ssr_mcp::{mcp_edit_page, mcp_servers_page};
pub(crate) use ssr_my_agents::{my_agent_edit_page, my_agents_page};
pub(crate) use ssr_my_hooks::{my_hook_edit_page, my_hooks_page};
pub(crate) use ssr_my_marketplace::my_marketplace_page;
pub(crate) use ssr_my_mcp::{my_mcp_edit_page, my_mcp_servers_page};
pub(crate) use ssr_my_plugin_view::my_plugin_view_page;
pub(crate) use ssr_my_plugins::{my_plugin_edit_page, my_plugins_page};
pub(crate) use ssr_my_secrets::my_secrets_page;
pub(crate) use ssr_my_skills::{my_skill_edit_page, my_skills_page};
pub(crate) use ssr_org_marketplace_edit::org_marketplace_edit_page;
pub(crate) use ssr_org_marketplaces::org_marketplaces_page;
pub(crate) use ssr_plugin_edit::{plugin_create_page, plugin_edit_page};
pub(crate) use ssr_plugins::plugins_page;
pub(crate) use ssr_skills::{skill_edit_page, skills_page};
pub(crate) use ssr_users::{user_detail_page, users_page};

pub(crate) async fn login_page(Extension(engine): Extension<AdminTemplateEngine>) -> Response {
    match engine.render("login", &json!({})) {
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

fn render_page(
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
                "department": user_ctx.department,
                "is_admin": user_ctx.is_admin,
            }),
        );
        let github_repo = "systempromptio/systemprompt-enterprise-demo-marketplace";
        let github_url = format!("https://github.com/{github_repo}");
        let install_cmd = format!("claude plugin marketplace add {github_repo}");
        let plugin_token = crate::admin::repositories::export_auth::generate_plugin_token(
            &user_ctx.user_id,
            &user_ctx.username,
            &user_ctx.email,
        )
        .unwrap_or_default();
        obj.insert(
            "marketplace".to_string(),
            json!({
                "user_id": mkt_ctx.user_id,
                "site_url": mkt_ctx.site_url,
                "total_plugins": mkt_ctx.total_plugins,
                "total_skills": mkt_ctx.total_skills,
                "agents_count": mkt_ctx.agents_count,
                "mcp_count": mkt_ctx.mcp_count,
                "github_url": github_url,
                "install_cmd": install_cmd,
                "plugin_token": plugin_token,
            }),
        );
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

fn get_services_path() -> Result<std::path::PathBuf, Box<Response>> {
    super::resources::get_services_path()
}
