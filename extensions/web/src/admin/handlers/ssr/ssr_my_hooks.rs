use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::Response,
};
use serde_json::json;
use sqlx::PgPool;

pub(crate) async fn my_hooks_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let hooks = repositories::list_user_hooks(&pool, &user_ctx.user_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list user hooks");
            vec![]
        });

    let hook_count = hooks.len();
    let enabled_count = hooks.iter().filter(|h| h.enabled).count();

    let hooks_json: Vec<serde_json::Value> = hooks
        .iter()
        .map(|h| {
            json!({
                "id": h.id,
                "hook_id": h.hook_id,
                "name": h.name,
                "description": h.description,
                "event": h.event,
                "matcher": h.matcher,
                "command": h.command,
                "is_async": h.is_async,
                "enabled": h.enabled,
                "base_hook_id": h.base_hook_id,
                "is_forked": h.base_hook_id.is_some(),
                "created_at": h.created_at,
                "updated_at": h.updated_at,
            })
        })
        .collect();

    let data = json!({
        "page": "my-hooks",
        "title": "My Hooks",
        "hooks": hooks_json,
        "hook_count": hook_count,
        "enabled_count": enabled_count,
        "hook_events": [
            "PostToolUse", "PostToolUseFailure", "PreToolUse",
            "UserPromptSubmit", "SessionStart", "SessionEnd",
            "Stop", "SubagentStart", "SubagentStop", "Notification"
        ],
    });
    super::render_page(&engine, "my-hooks", &data, &user_ctx, &mkt_ctx)
}

pub(crate) async fn my_hook_edit_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let hook_id = params.get("id");
    let is_edit = hook_id.is_some();

    let hook = if let Some(id) = hook_id {
        let hooks = repositories::list_user_hooks(&pool, &user_ctx.user_id)
            .await
            .unwrap_or_default();
        hooks.into_iter().find(|h| h.hook_id == *id)
    } else {
        None
    };

    let is_forked = hook.as_ref().is_some_and(|h| h.base_hook_id.is_some());

    let data = json!({
        "page": "my-hook-edit",
        "title": if is_edit { "Edit My Hook" } else { "Create My Hook" },
        "is_edit": is_edit,
        "hook": hook,
        "is_forked": is_forked,
        "hook_events": [
            "PostToolUse", "PostToolUseFailure", "PreToolUse",
            "UserPromptSubmit", "SessionStart", "SessionEnd",
            "Stop", "SubagentStart", "SubagentStop", "Notification"
        ],
    });
    super::render_page(&engine, "my-hook-edit", &data, &user_ctx, &mkt_ctx)
}
