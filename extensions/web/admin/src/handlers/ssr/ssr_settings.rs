use std::sync::Arc;

use crate::repositories;
use crate::repositories::subscriptions as sub_repo;
use crate::repositories::user_settings::UserSettingsRow;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, State},
    response::Response,
};
use sqlx::PgPool;

use super::types::{SettingsPageData, SettingsView, UsageItemView};

fn format_limit(val: Option<i64>) -> String {
    match val {
        Some(v) if v < 0 => "\u{221e}".to_string(),
        Some(v) => v.to_string(),
        None => "\u{221e}".to_string(),
    }
}

const FREE_TIER_LIMITS: &str = r#"{"max_plugins":1,"max_skills":5,"max_agents":1,"max_mcp_servers":1,"max_hooks":5,"max_secrets":2}"#;

pub async fn settings_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let (settings, current_limits, hooks_count, secrets_count) =
        fetch_settings_data(&pool, &user_ctx).await;

    let usage_items = build_usage_items(&mkt_ctx, &current_limits, hooks_count, secrets_count);

    let settings_view = settings.as_ref().map_or_else(
        || SettingsView {
            display_name: None,
            avatar_url: None,
            notify_daily_summary: true,
            notify_achievements: true,
            leaderboard_opt_in: true,
            timezone: "UTC".to_string(),
        },
        |s| SettingsView {
            display_name: s.display_name.clone(),
            avatar_url: s.avatar_url.clone(),
            notify_daily_summary: s.notify_daily_summary,
            notify_achievements: s.notify_achievements,
            leaderboard_opt_in: s.leaderboard_opt_in,
            timezone: s.timezone.clone(),
        },
    );

    let data = SettingsPageData {
        page: "settings",
        title: "Account Settings",
        settings: settings_view,
        user_email: user_ctx.email.to_string(),
        user_id: user_ctx.user_id.to_string(),
        username: user_ctx.username.clone(),
        tier_name: mkt_ctx.tier_name.clone(),
        is_premium: mkt_ctx.is_premium,
        usage_items,
    };

    let value = serde_json::to_value(&data).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to serialize settings page data");
        serde_json::Value::Null
    });
    super::render_page(&engine, "settings", &value, &user_ctx, &mkt_ctx)
}

async fn fetch_settings_data(
    pool: &PgPool,
    user_ctx: &UserContext,
) -> (Option<UserSettingsRow>, serde_json::Value, usize, usize) {
    let (settings_result, sub_result, plans_result) = tokio::join!(
        repositories::user_settings::find_user_settings(pool, &user_ctx.user_id),
        sub_repo::find_subscription_by_user(pool, &user_ctx.user_id),
        sub_repo::list_active_plans(pool),
    );

    let settings = settings_result.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to get user settings");
        None
    });
    let subscription = sub_result.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to get subscription for settings");
        None
    });
    let plans = plans_result.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list plans for settings");
        vec![]
    });

    let (hooks_result, secrets_result) = tokio::join!(
        repositories::user_hooks::list_user_hooks(pool, &user_ctx.user_id),
        repositories::list_all_user_env_vars(pool, &user_ctx.user_id),
    );

    let hooks_count = hooks_result.as_ref().map_or(0, Vec::len);
    let secrets_count = secrets_result.as_ref().map_or(0, Vec::len);

    let sub_plan_id = subscription.as_ref().and_then(|s| s.plan_id);
    let current_limits = sub_plan_id
        .and_then(|pid| plans.iter().find(|p| p.id == pid))
        .map_or_else(
            || {
                serde_json::from_str(FREE_TIER_LIMITS).unwrap_or_else(|e| {
                    tracing::warn!(error = %e, "Failed to parse free tier limits");
                    serde_json::json!({})
                })
            },
            |p| p.limits.clone(),
        );

    (settings, current_limits, hooks_count, secrets_count)
}

fn build_usage_items(
    mkt_ctx: &MarketplaceContext,
    current_limits: &serde_json::Value,
    hooks_count: usize,
    secrets_count: usize,
) -> Vec<UsageItemView> {
    let get_limit =
        |key: &str| -> Option<i64> { current_limits.get(key).and_then(serde_json::Value::as_i64) };

    vec![
        build_usage_item("Plugins", mkt_ctx.total_plugins, get_limit("max_plugins")),
        build_usage_item("Skills", mkt_ctx.total_skills, get_limit("max_skills")),
        build_usage_item("Agents", mkt_ctx.agents_count, get_limit("max_agents")),
        build_usage_item(
            "MCP Servers",
            mkt_ctx.mcp_count,
            get_limit("max_mcp_servers"),
        ),
        build_usage_item("Hooks", hooks_count, get_limit("max_hooks")),
        build_usage_item("Secrets", secrets_count, get_limit("max_secrets")),
    ]
}

fn build_usage_item(label: &str, current: usize, max: Option<i64>) -> UsageItemView {
    let limit_display = format_limit(max);
    let is_unlimited = max.is_none_or(|v| v < 0);
    let percentage = if is_unlimited {
        0
    } else {
        let max_val = usize::try_from(max.unwrap_or(1)).unwrap_or(1);
        if max_val == 0 {
            100
        } else {
            usize::min(current.saturating_mul(100) / max_val, 100)
        }
    };
    let is_at_limit =
        !is_unlimited && max.is_some_and(|m| i64::try_from(current).unwrap_or(i64::MAX) >= m);

    UsageItemView {
        label: label.to_string(),
        current,
        limit_display,
        is_unlimited,
        percentage,
        is_at_limit,
    }
}
