use std::sync::Arc;

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;

use crate::admin::activity::{self, ActivityEntity, NewActivity};
use crate::admin::handlers::shared;
use crate::admin::repositories;
use crate::admin::types::{
    CreatePluginRequest, UpdatePluginRequest, UpdateSkillFileRequest, UserContext,
};

use super::resources::get_services_path;
use super::responses::FilesListResponse;

pub(crate) async fn get_plugin_detail_handler(Path(plugin_id): Path<String>) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match repositories::find_plugin_detail(&services_path, &plugin_id) {
        Ok(Some(plugin)) => Json(plugin).into_response(),
        Ok(None) => shared::error_response(StatusCode::NOT_FOUND, "Plugin not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get plugin");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

pub(crate) async fn create_plugin_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Json(body): Json<CreatePluginRequest>,
) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match repositories::create_plugin(&services_path, &body) {
        Ok(plugin) => {
            let name = body.name.clone();
            let pid = plugin.id.clone();
            let uid = user_ctx.user_id.clone();
            tokio::spawn(async move {
                activity::record(
                    &pool,
                    NewActivity::entity_created(&uid, ActivityEntity::Plugin, &pid, &name),
                )
                .await;
            });
            (StatusCode::CREATED, Json(plugin)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create plugin");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

pub(crate) async fn update_plugin_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(plugin_id): Path<String>,
    Json(body): Json<UpdatePluginRequest>,
) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match repositories::update_plugin(&services_path, &plugin_id, &body) {
        Ok(Some(plugin)) => {
            let name = plugin.name.clone();
            let pid = plugin_id.clone();
            let uid = user_ctx.user_id.clone();
            tokio::spawn(async move {
                activity::record(
                    &pool,
                    NewActivity::entity_updated(&uid, ActivityEntity::Plugin, &pid, &name),
                )
                .await;
            });
            Json(plugin).into_response()
        }
        Ok(None) => shared::error_response(StatusCode::NOT_FOUND, "Plugin not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to update plugin");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

pub(crate) async fn delete_plugin_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(plugin_id): Path<String>,
) -> Response {
    if !user_ctx.is_admin {
        return shared::error_response(StatusCode::FORBIDDEN, "Admin access required");
    }
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match repositories::delete_plugin(&services_path, &plugin_id) {
        Ok(true) => {
            let pid = plugin_id.clone();
            let uid = user_ctx.user_id.clone();
            tokio::spawn(async move {
                activity::record(
                    &pool,
                    NewActivity::entity_deleted(&uid, ActivityEntity::Plugin, &pid, &pid),
                )
                .await;
            });
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(false) => shared::error_response(StatusCode::NOT_FOUND, "Plugin not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to delete plugin");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

pub(crate) async fn list_skill_files_handler(
    State(pool): State<Arc<PgPool>>,
    Path(skill_id): Path<String>,
) -> Response {
    match repositories::list_skill_files(&pool, &skill_id).await {
        Ok(files) => Json(FilesListResponse { files }).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list skill files");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

pub(crate) async fn get_skill_file_handler(
    State(pool): State<Arc<PgPool>>,
    Path((skill_id, file_path)): Path<(String, String)>,
) -> Response {
    match repositories::find_skill_file(&pool, &skill_id, &file_path).await {
        Ok(Some(file)) => Json(file).into_response(),
        Ok(None) => shared::error_response(StatusCode::NOT_FOUND, "File not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get skill file");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

pub(crate) async fn update_skill_file_handler(
    State(pool): State<Arc<PgPool>>,
    Path((skill_id, file_path)): Path<(String, String)>,
    Json(body): Json<UpdateSkillFileRequest>,
) -> Response {
    let services_path = match shared::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match repositories::update_skill_file_content(
        &pool,
        &skill_id,
        &file_path,
        &body.content,
        &services_path,
    )
    .await
    {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => shared::error_response(StatusCode::NOT_FOUND, "File not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to update skill file");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

pub(crate) async fn sync_skill_files_handler(State(pool): State<Arc<PgPool>>) -> Response {
    let services_path = match shared::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match repositories::sync_skill_files(&pool, &services_path).await {
        Ok(result) => Json(result).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to sync skill files");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}
