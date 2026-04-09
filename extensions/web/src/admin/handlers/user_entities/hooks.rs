use std::sync::Arc;

use crate::admin::activity::{self, ActivityEntity, NewActivity};
use crate::admin::repositories;
use crate::admin::types::{CreateUserHookRequest, UpdateUserHookRequest, UserContext};
use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use sqlx::PgPool;

use crate::admin::handlers::{responses::HooksListResponse, shared};

#[derive(Debug, Deserialize)]
pub struct CreateHookApiRequest {
    pub hook_name: String,
    #[serde(default)]
    pub description: String,
    pub event_type: String,
    #[serde(default = "default_matcher")]
    pub matcher: String,
    #[serde(default = "default_hook_type")]
    pub hook_type: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub headers: serde_json::Value,
    #[serde(default = "default_timeout")]
    pub timeout: i32,
    #[serde(default)]
    pub is_async: bool,
    pub plugin_id: Option<String>,
}

fn default_matcher() -> String {
    "*".to_string()
}
fn default_hook_type() -> String {
    "http".to_string()
}
const fn default_timeout() -> i32 {
    10
}

#[derive(Debug, Deserialize)]
pub struct UpdateHookApiRequest {
    pub hook_name: Option<String>,
    pub description: Option<String>,
    pub event_type: Option<String>,
    pub matcher: Option<String>,
    pub url: Option<String>,
    pub command: Option<String>,
    pub headers: Option<serde_json::Value>,
    pub timeout: Option<i32>,
    pub is_async: Option<bool>,
    pub enabled: Option<bool>,
}

pub async fn list_user_hooks_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    match repositories::user_hooks::list_user_hooks(&pool, &user_ctx.user_id).await {
        Ok(hooks) => Json(HooksListResponse { hooks }).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list user hooks");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to list hooks")
        }
    }
}

pub async fn create_user_hook_handler(
    Extension(user_ctx): Extension<UserContext>,
    Extension(tier_cache): Extension<crate::admin::tier_enforcement::TierEnforcementCache>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<CreateHookApiRequest>,
) -> Response {
    let limit_check = crate::admin::tier_enforcement::check_limit(
        &tier_cache,
        pool.as_ref(),
        &user_ctx.user_id,
        crate::admin::tier_limits::LimitCheck::CreateHook,
    )
    .await;
    if !limit_check.allowed {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "entity_limit_reached",
                "entity_type": "hook",
                "message": limit_check.reason,
                "limit": limit_check.limit_value,
                "current": limit_check.current_value,
            })),
        )
            .into_response();
    }

    let create_req = CreateUserHookRequest {
        hook_name: req.hook_name,
        description: req.description,
        event_type: req.event_type,
        matcher: req.matcher,
        hook_type: req.hook_type,
        url: req.url,
        command: req.command,
        headers: req.headers,
        timeout: req.timeout,
        is_async: req.is_async,
        plugin_id: req.plugin_id,
    };
    match repositories::user_hooks::create_user_hook(&pool, &user_ctx.user_id, &create_req).await {
        Ok(hook) => {
            if let Err(e) = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await {
                tracing::warn!(error = %e, "Failed to mark user dirty");
            }
            let activity_pool = Arc::clone(&pool);
            let uid = user_ctx.user_id.clone();
            let id = hook.id.clone();
            let name = hook.hook_name.clone();
            tokio::spawn(async move {
                activity::record(
                    &activity_pool,
                    NewActivity::entity_created(&uid, ActivityEntity::Hook, &id, &name),
                )
                .await;
            });
            StatusCode::CREATED.into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create hook");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create hook")
        }
    }
}

pub async fn update_user_hook_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(hook_id): Path<String>,
    Json(req): Json<UpdateHookApiRequest>,
) -> Response {
    let update_req = UpdateUserHookRequest {
        hook_name: req.hook_name,
        description: req.description,
        event_type: req.event_type,
        matcher: req.matcher,
        url: req.url,
        command: req.command,
        headers: req.headers,
        timeout: req.timeout,
        is_async: req.is_async,
        enabled: req.enabled,
    };
    match repositories::user_hooks::update_user_hook(
        &pool,
        &hook_id,
        &user_ctx.user_id,
        &update_req,
    )
    .await
    {
        Ok(Some(_)) => {
            if let Err(e) = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await {
                tracing::warn!(error = %e, "Failed to mark user dirty");
            }
            let activity_pool = Arc::clone(&pool);
            let uid = user_ctx.user_id.clone();
            let id = hook_id.clone();
            tokio::spawn(async move {
                activity::record(
                    &activity_pool,
                    NewActivity::entity_updated(&uid, ActivityEntity::Hook, &id, &id),
                )
                .await;
            });
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(None) => shared::error_response(StatusCode::NOT_FOUND, "Hook not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to update hook");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to update hook")
        }
    }
}

pub async fn delete_user_hook_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(hook_id): Path<String>,
) -> Response {
    match repositories::user_hooks::delete_user_hook(&pool, &hook_id, &user_ctx.user_id).await {
        Ok(true) => {
            if let Err(e) = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await {
                tracing::warn!(error = %e, "Failed to mark user dirty");
            }
            let activity_pool = Arc::clone(&pool);
            let uid = user_ctx.user_id.clone();
            let id = hook_id.clone();
            tokio::spawn(async move {
                activity::record(
                    &activity_pool,
                    NewActivity::entity_deleted(&uid, ActivityEntity::Hook, &id, &id),
                )
                .await;
            });
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(false) => shared::error_response(StatusCode::NOT_FOUND, "Hook not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to delete hook");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete hook")
        }
    }
}

pub async fn toggle_user_hook_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(hook_id): Path<String>,
) -> Response {
    match repositories::user_hooks::toggle_user_hook(&pool, &hook_id, &user_ctx.user_id).await {
        Ok(Some(_)) => {
            if let Err(e) = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await {
                tracing::warn!(error = %e, "Failed to mark user dirty");
            }
            let activity_pool = Arc::clone(&pool);
            let uid = user_ctx.user_id.clone();
            let id = hook_id.clone();
            tokio::spawn(async move {
                activity::record(
                    &activity_pool,
                    NewActivity::entity_updated(&uid, ActivityEntity::Hook, &id, &id),
                )
                .await;
            });
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(None) => shared::error_response(StatusCode::NOT_FOUND, "Hook not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to toggle hook");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to toggle hook")
        }
    }
}
