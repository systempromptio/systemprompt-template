use crate::admin::numeric;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, UserContext};
use crate::utils::html_escape;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde_json::json;

use super::ssr_demo_help::demo_help_text;

pub fn branding_context(engine: &AdminTemplateEngine) -> serde_json::Value {
    match engine.branding() {
        Some(b) => json!({"branding": b}),
        None => json!({}),
    }
}

pub fn render_page(
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
        let mcp_url = format!("{}/api/v1/mcp/skill-manager/mcp", mkt_ctx.site_url);
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
                "plugin_token": mkt_ctx.plugin_token,
            }),
        );
        obj.entry("page_stats".to_string())
            .or_insert_with(|| json!([]));
        if let Some(branding) = engine.branding() {
            if let Ok(val) = serde_json::to_value(branding) {
                obj.insert("branding".to_string(), val);
            }
        }
        if let Some(page_str) = obj.get("page").and_then(|v| v.as_str()) {
            if let Some((help, doc_slug)) = demo_help_text(page_str) {
                obj.insert("demo_help".to_string(), json!(help));
                obj.insert(
                    "demo_help_url".to_string(),
                    json!(format!("/documentation/{}", doc_slug)),
                );
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
