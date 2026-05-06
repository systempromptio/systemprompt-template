use crate::numeric;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde_json::json;
use systemprompt_web_shared::html_escape;

use super::ssr_demo_help::demo_help_text;

pub fn branding_context(engine: &AdminTemplateEngine) -> serde_json::Value {
    engine
        .branding()
        .map_or_else(|| json!({}), |b| json!({"branding": b}))
}

pub fn render_typed_page<T: serde::Serialize>(
    engine: &AdminTemplateEngine,
    template: &str,
    data: &T,
    user_ctx: &UserContext,
    mkt_ctx: &MarketplaceContext,
) -> Response {
    let value = serde_json::to_value(data).unwrap_or_else(|e| {
        tracing::warn!(template, error = %e, "Failed to serialize SSR page data");
        serde_json::Value::Object(serde_json::Map::new())
    });
    render_page(engine, template, &value, user_ctx, mkt_ctx)
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
        inject_user_and_marketplace(obj, engine, user_ctx, mkt_ctx);
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

fn inject_user_and_marketplace(
    obj: &mut serde_json::Map<String, serde_json::Value>,
    engine: &AdminTemplateEngine,
    user_ctx: &UserContext,
    mkt_ctx: &MarketplaceContext,
) {
    obj.insert(
        "current_user".to_string(),
        json!({
            "user_id": user_ctx.user_id,
            "username": user_ctx.username,
            "roles": user_ctx.roles,
            "is_admin": user_ctx.is_admin,
        }),
    );
    obj.insert("marketplace".to_string(), build_marketplace_json(mkt_ctx));
    obj.entry("page_stats".to_string())
        .or_insert_with(|| json!([]));
    if let Some(branding) = engine.branding() {
        if let Ok(val) = serde_json::to_value(branding) {
            obj.insert("branding".to_string(), val);
        }
    }
    if let Some(page_str) = obj.get("page").and_then(|v| v.as_str()) {
        let (help, doc_slug) = demo_help_text(page_str);
        obj.insert("demo_help".to_string(), json!(help));
        obj.insert(
            "demo_help_url".to_string(),
            json!(format!("/documentation/{}", doc_slug)),
        );
    }
}

fn build_marketplace_json(mkt_ctx: &MarketplaceContext) -> serde_json::Value {
    json!({
        "user_id": mkt_ctx.user_id,
        "site_url": mkt_ctx.site_url,
        "total_plugins": mkt_ctx.total_plugins,
        "total_skills": mkt_ctx.total_skills,
        "agents_count": mkt_ctx.agents_count,
        "mcp_count": mkt_ctx.mcp_count,
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
    })
}
