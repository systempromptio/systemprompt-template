use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::handlers::shared;
use crate::repositories;

use super::responses::SkillIdsListResponse;

pub async fn list_all_skills_handler() -> Response {
    let services_path = match shared::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    match repositories::list_all_skill_ids(&services_path) {
        Ok(ids) => Json(SkillIdsListResponse { skill_ids: ids }).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list skills");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
        }
    }
}

pub async fn get_plugin_skills_handler(Path(plugin_id): Path<String>) -> Response {
    let services_path = match shared::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    match repositories::list_plugin_skill_ids(&services_path, &plugin_id) {
        Ok(ids) => Json(SkillIdsListResponse { skill_ids: ids }).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get plugin skills");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
        }
    }
}
