use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;
use systemprompt::models::ProfileBootstrap;

use crate::admin::activity::{self, ActivityEntity, NewActivity};
use crate::admin::repositories;
use crate::admin::types::{UpdatePluginSkillsRequest, UpdateSkillRequest, UserQuery};

pub(crate) async fn update_skill_handler(
    State(pool): State<Arc<PgPool>>,
    Path(skill_id): Path<String>,
    Json(body): Json<UpdateSkillRequest>,
) -> Response {
    let Some(enabled) = body.enabled else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "enabled field required"})),
        )
            .into_response();
    };
    match repositories::update_agent_skill_enabled(&pool, &skill_id, enabled).await {
        Ok(Some(skill)) => {
            let sid = skill_id.clone();
            let sname = skill.name.clone();
            let p = pool.clone();
            tokio::spawn(async move {
                activity::record(
                    &p,
                    NewActivity::entity_updated("admin", ActivityEntity::Skill, &sid, &sname),
                )
                .await;
            });
            Json(skill).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Skill not found"})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to update skill");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn delete_skill_handler(
    State(pool): State<Arc<PgPool>>,
    Path(skill_id): Path<String>,
    Query(query): Query<UserQuery>,
) -> Response {
    let user_id = query.user_id.as_deref().unwrap_or("admin");
    match repositories::delete_user_skill(&pool, user_id, &skill_id).await {
        Ok(true) => {
            let sid = skill_id.clone();
            let uid = user_id.to_string();
            let p = pool.clone();
            tokio::spawn(async move {
                activity::record(
                    &p,
                    NewActivity::entity_deleted(&uid, ActivityEntity::Skill, &sid, &sid),
                )
                .await;
            });
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Skill not found"})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to delete skill");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn list_all_skills_handler() -> Response {
    let services_path = match ProfileBootstrap::get() {
        Ok(profile) => std::path::PathBuf::from(&profile.paths.services),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get profile bootstrap");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to load profile"})),
            )
                .into_response();
        }
    };

    match repositories::list_all_skill_ids(&services_path) {
        Ok(ids) => Json(ids).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list skills");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn get_plugin_skills_handler(Path(plugin_id): Path<String>) -> Response {
    let services_path = match ProfileBootstrap::get() {
        Ok(profile) => std::path::PathBuf::from(&profile.paths.services),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get profile bootstrap");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to load profile"})),
            )
                .into_response();
        }
    };

    match repositories::get_plugin_skill_ids(&services_path, &plugin_id) {
        Ok(ids) => Json(ids).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get plugin skills");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn update_plugin_skills_handler(
    Path(plugin_id): Path<String>,
    Json(body): Json<UpdatePluginSkillsRequest>,
) -> Response {
    let services_path = match ProfileBootstrap::get() {
        Ok(profile) => std::path::PathBuf::from(&profile.paths.services),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get profile bootstrap");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to load profile"})),
            )
                .into_response();
        }
    };

    match repositories::update_plugin_skills(&services_path, &plugin_id, &body.skills) {
        Ok(()) => Json(serde_json::json!({"ok": true})).into_response(),
        Err(e) => {
            tracing::error!(error = %e, plugin_id = %plugin_id, "Failed to update plugin skills");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}
