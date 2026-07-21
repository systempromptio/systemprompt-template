//! Context fragments injected into every admin page render.

use crate::error::AdminHtmlError;
use crate::numeric;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::response::{Html, IntoResponse, Response};
use serde::Serialize;
use serde_json::json;
use systemprompt_web_shared::{RankTier, UserId};

use super::ssr_demo_help::demo_help_text;

#[derive(Debug, Serialize)]
struct CurrentUser<'a> {
    user_id: &'a UserId,
    username: &'a str,
    roles: &'a [String],
    is_admin: bool,
}

#[derive(Debug, Serialize)]
struct MarketplaceView<'a> {
    user_id: &'a UserId,
    site_url: &'a str,
    total_plugins: usize,
    total_skills: usize,
    agents_count: usize,
    mcp_count: usize,
    rank_level: i32,
    rank_name: &'a str,
    rank_tier: RankTier,
    total_xp: i64,
    xp_progress_pct: i64,
    has_completed_onboarding: bool,
    current_streak: i64,
    longest_streak: i64,
    next_rank_name: &'a str,
    xp_to_next_rank: i64,
}

pub(crate) fn branding_context(engine: &AdminTemplateEngine) -> serde_json::Value {
    engine
        .branding()
        .map_or_else(|| json!({}), |b| json!({"branding": b}))
}

pub(crate) fn render_typed_page<T: Serialize>(
    engine: &AdminTemplateEngine,
    template: &str,
    data: &T,
    user_ctx: &UserContext,
    mkt_ctx: &MarketplaceContext,
) -> Response {
    // lint-ok: http-error — renders a page, and its failure arm is AdminHtmlError
    let value = serde_json::to_value(data).unwrap_or_else(|e| {
        tracing::warn!(template, error = %e, "Failed to serialize SSR page data");
        serde_json::Value::Object(serde_json::Map::new())
    });
    render_page(engine, template, &value, user_ctx, mkt_ctx)
}

pub(crate) fn render_page(
    engine: &AdminTemplateEngine,
    template: &str,
    data: &serde_json::Value,
    user_ctx: &UserContext,
    mkt_ctx: &MarketplaceContext,
) -> Response {
    // lint-ok: http-error — renders a page, and its failure arm is AdminHtmlError
    let mut merged = data.clone();
    if let Some(obj) = merged.as_object_mut() {
        inject_user_and_marketplace(obj, engine, user_ctx, mkt_ctx);
    }
    match engine.render(template, &merged) {
        Ok(html) => Html(html).into_response(),
        Err(e) => AdminHtmlError::internal(format!("SSR render failed for {template}: {e:?}"))
            .into_response(),
    }
}

fn inject_user_and_marketplace(
    obj: &mut serde_json::Map<String, serde_json::Value>,
    engine: &AdminTemplateEngine,
    user_ctx: &UserContext,
    mkt_ctx: &MarketplaceContext,
) {
    let current_user = CurrentUser {
        user_id: &user_ctx.user_id,
        username: &user_ctx.username,
        roles: &user_ctx.roles,
        is_admin: user_ctx.is_admin,
    };
    obj.insert(
        "current_user".to_owned(),
        serde_json::to_value(current_user).unwrap_or(serde_json::Value::Null),
    );
    obj.insert("marketplace".to_owned(), build_marketplace_json(mkt_ctx));
    obj.entry("page_stats".to_owned())
        .or_insert_with(|| json!([]));
    if let Some(branding) = engine.branding()
        && let Ok(val) = serde_json::to_value(branding)
    {
        obj.insert("branding".to_owned(), val);
    }
    if let Some(page_str) = obj.get("page").and_then(|v| v.as_str()) {
        let (help, doc_slug) = demo_help_text(page_str);
        obj.insert("demo_help".to_owned(), json!(help));
        obj.insert(
            "demo_help_url".to_owned(),
            json!(format!("/documentation/{}", doc_slug)),
        );
    }
}

fn build_marketplace_json(mkt_ctx: &MarketplaceContext) -> serde_json::Value {
    let view = MarketplaceView {
        user_id: &mkt_ctx.user_id,
        site_url: &mkt_ctx.site_url,
        total_plugins: mkt_ctx.total_plugins,
        total_skills: mkt_ctx.total_skills,
        agents_count: mkt_ctx.agents_count,
        mcp_count: mkt_ctx.mcp_count,
        rank_level: mkt_ctx.rank_level,
        rank_name: &mkt_ctx.rank_name,
        rank_tier: mkt_ctx.rank_tier,
        total_xp: mkt_ctx.total_xp,
        xp_progress_pct: numeric::round_to_i64(mkt_ctx.xp_progress_pct),
        has_completed_onboarding: mkt_ctx.has_completed_onboarding,
        current_streak: mkt_ctx.current_streak,
        longest_streak: mkt_ctx.longest_streak,
        next_rank_name: &mkt_ctx.next_rank_name,
        xp_to_next_rank: mkt_ctx.xp_to_next_rank,
    };
    serde_json::to_value(view).unwrap_or(serde_json::Value::Null)
}
