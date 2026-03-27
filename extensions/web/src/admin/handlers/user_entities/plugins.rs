use std::sync::Arc;

use crate::admin::activity::{self, ActivityEntity, NewActivity};
use crate::admin::repositories;
use crate::admin::types::{CreateUserPluginRequest, UpdateUserPluginRequest, UserContext};
use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use sqlx::PgPool;

use systemprompt::identifiers::{AgentId, SkillId, UserId};

use crate::admin::handlers::{responses::PluginsListResponse, shared};

async fn is_platform_plugin(pool: &Arc<PgPool>, user_id: &UserId, plugin_id: &str) -> bool {
    repositories::find_user_plugin(pool, user_id, plugin_id)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, user_id = %user_id.as_str(), plugin_id = %plugin_id, "Failed to check if plugin is platform plugin");
        })
        .ok()
        .flatten()
        .is_some_and(|p| p.base_plugin_id.as_deref() == Some("systemprompt"))
}

pub(crate) async fn is_entity_in_platform_plugin(
    pool: &Arc<PgPool>,
    user_id: &UserId,
    entity_id: &str,
    entity_kind: &str,
) -> bool {
    repositories::is_entity_in_platform_plugin(pool, user_id, entity_id, entity_kind).await
}

pub(crate) async fn list_user_plugins_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    match repositories::list_user_plugins(&pool, &user_ctx.user_id).await {
        Ok(plugins) => Json(PluginsListResponse { plugins }).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list user plugins");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to list plugins")
        }
    }
}

pub(crate) async fn create_user_plugin_handler(
    Extension(user_ctx): Extension<UserContext>,
    Extension(tier_cache): Extension<crate::admin::tier_enforcement::TierEnforcementCache>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<CreateUserPluginRequest>,
) -> Response {
    let limit_check = crate::admin::tier_enforcement::check_limit(
        &tier_cache,
        pool.as_ref(),
        &user_ctx.user_id,
        crate::admin::tier_limits::LimitCheck::CreatePlugin,
    )
    .await;
    if !limit_check.allowed {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "entity_limit_reached",
                "entity_type": "plugin",
                "message": limit_check.reason,
                "limit": limit_check.limit_value,
                "current": limit_check.current_value,
            })),
        )
            .into_response();
    }

    match repositories::create_user_plugin(&pool, &user_ctx.user_id, &req).await {
        Ok(plugin) => {
            if let Err(e) = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await {
                tracing::warn!(error = %e, "Failed to mark user dirty");
            }
            let activity_pool = pool.clone();
            let uid = user_ctx.user_id.clone();
            let id = plugin.id.clone();
            let name = plugin.name.clone();
            tokio::spawn(async move {
                activity::record(
                    &activity_pool,
                    NewActivity::entity_created(&uid, ActivityEntity::Plugin, &id, &name),
                )
                .await;
            });
            (StatusCode::CREATED, Json(plugin)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create user plugin");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create plugin")
        }
    }
}

pub(crate) async fn update_user_plugin_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
    Json(req): Json<UpdateUserPluginRequest>,
) -> Response {
    if is_platform_plugin(&pool, &user_ctx.user_id, &plugin_id).await {
        return shared::error_response(StatusCode::FORBIDDEN, "Platform plugin cannot be modified");
    }
    match repositories::update_user_plugin(&pool, &user_ctx.user_id, &plugin_id, &req).await {
        Ok(Some(plugin)) => {
            if let Err(e) = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await {
                tracing::warn!(error = %e, "Failed to mark user dirty");
            }
            let activity_pool = pool.clone();
            let uid = user_ctx.user_id.clone();
            let id = plugin.id.clone();
            let name = plugin.name.clone();
            tokio::spawn(async move {
                activity::record(
                    &activity_pool,
                    NewActivity::entity_updated(&uid, ActivityEntity::Plugin, &id, &name),
                )
                .await;
            });
            Json(plugin).into_response()
        }
        Ok(None) => shared::error_response(StatusCode::NOT_FOUND, "Plugin not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to update user plugin");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to update plugin")
        }
    }
}

pub(crate) async fn delete_user_plugin_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
) -> Response {
    if is_platform_plugin(&pool, &user_ctx.user_id, &plugin_id).await {
        return shared::error_response(StatusCode::FORBIDDEN, "Platform plugin cannot be modified");
    }
    match repositories::delete_user_plugin(&pool, &user_ctx.user_id, &plugin_id).await {
        Ok(true) => {
            if let Err(e) = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await {
                tracing::warn!(error = %e, "Failed to mark user dirty");
            }
            let activity_pool = pool.clone();
            let uid = user_ctx.user_id.clone();
            let id = plugin_id.clone();
            tokio::spawn(async move {
                activity::record(
                    &activity_pool,
                    NewActivity::entity_deleted(&uid, ActivityEntity::Plugin, &id, &id),
                )
                .await;
            });
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(false) => shared::error_response(StatusCode::NOT_FOUND, "Plugin not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to delete user plugin");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete plugin")
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct SetAssociationsRequest {
    pub ids: Vec<String>,
}

pub(crate) async fn set_plugin_skills_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
    Json(req): Json<SetAssociationsRequest>,
) -> Response {
    if is_platform_plugin(&pool, &user_ctx.user_id, &plugin_id).await {
        return shared::error_response(StatusCode::FORBIDDEN, "Platform plugin cannot be modified");
    }
    let plugin = repositories::find_user_plugin(&pool, &user_ctx.user_id, &plugin_id).await;
    match plugin {
        Ok(Some(p)) => {
            let skill_ids: Vec<SkillId> =
                req.ids.iter().map(|s| SkillId::from(s.clone())).collect();
            if let Err(e) = repositories::set_plugin_skills(&pool, &p.id, &skill_ids).await {
                tracing::error!(error = %e, "Failed to set plugin skills");
                return shared::error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to set skills",
                );
            }
            if let Err(e) = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await {
                tracing::warn!(error = %e, "Failed to mark user dirty");
            }
            let activity_pool = pool.clone();
            let uid = user_ctx.user_id.clone();
            let pid = plugin_id.clone();
            tokio::spawn(async move {
                activity::record(
                    &activity_pool,
                    NewActivity::entity_updated(&uid, ActivityEntity::Plugin, &pid, &pid),
                )
                .await;
            });
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(None) => shared::error_response(StatusCode::NOT_FOUND, "Plugin not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get user plugin");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
        }
    }
}

pub(crate) async fn set_plugin_agents_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
    Json(req): Json<SetAssociationsRequest>,
) -> Response {
    if is_platform_plugin(&pool, &user_ctx.user_id, &plugin_id).await {
        return shared::error_response(StatusCode::FORBIDDEN, "Platform plugin cannot be modified");
    }
    let plugin = repositories::find_user_plugin(&pool, &user_ctx.user_id, &plugin_id).await;
    match plugin {
        Ok(Some(p)) => {
            let agent_ids: Vec<AgentId> =
                req.ids.iter().map(|s| AgentId::from(s.clone())).collect();
            if let Err(e) = repositories::set_plugin_agents(&pool, &p.id, &agent_ids).await {
                tracing::error!(error = %e, "Failed to set plugin agents");
                return shared::error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to set agents",
                );
            }
            if let Err(e) = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await {
                tracing::warn!(error = %e, "Failed to mark user dirty");
            }
            let activity_pool = pool.clone();
            let uid = user_ctx.user_id.clone();
            let pid = plugin_id.clone();
            tokio::spawn(async move {
                activity::record(
                    &activity_pool,
                    NewActivity::entity_updated(&uid, ActivityEntity::Plugin, &pid, &pid),
                )
                .await;
            });
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(None) => shared::error_response(StatusCode::NOT_FOUND, "Plugin not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get user plugin");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
        }
    }
}
