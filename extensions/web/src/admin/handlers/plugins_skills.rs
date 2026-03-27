use std::sync::Arc;

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;

use systemprompt::identifiers::{SkillId, UserId};

use crate::admin::activity::{self, ActivityEntity, NewActivity};
use crate::admin::handlers::shared;
use crate::admin::repositories;
use crate::admin::types::{UpdatePluginSkillsRequest, UserContext, UserQuery};

use super::responses::SkillIdsListResponse;

pub(crate) async fn delete_skill_handler(
    State(pool): State<Arc<PgPool>>,
    Path(skill_id_raw): Path<String>,
    Query(query): Query<UserQuery>,
) -> Response {
    let default_user_id = UserId::new("admin");
    let user_id = query.user_id.as_ref().map_or_else(|| default_user_id.clone(), UserId::new);
    let skill_id = SkillId::new(skill_id_raw);
    match repositories::delete_user_skill(&pool, &user_id, &skill_id).await {
        Ok(true) => {
            let sid = skill_id.clone();
            let uid = user_id.clone();
            let p = pool.clone();
            tokio::spawn(async move {
                activity::record(
                    &p,
                    NewActivity::entity_deleted(
                        &uid,
                        ActivityEntity::Skill,
                        sid.as_str(),
                        sid.as_str(),
                    ),
                )
                .await;
            });
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(false) => shared::error_response(StatusCode::NOT_FOUND, "Skill not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to delete skill");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

pub(crate) async fn list_all_skills_handler() -> Response {
    let services_path = match shared::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    match repositories::list_all_skill_ids(&services_path) {
        Ok(ids) => Json(SkillIdsListResponse { skill_ids: ids }).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list skills");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

pub(crate) async fn get_plugin_skills_handler(Path(plugin_id): Path<String>) -> Response {
    let services_path = match shared::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    match repositories::get_plugin_skill_ids(&services_path, &plugin_id) {
        Ok(ids) => Json(SkillIdsListResponse { skill_ids: ids }).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get plugin skills");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

pub(crate) async fn update_plugin_skills_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(plugin_id): Path<String>,
    Json(body): Json<UpdatePluginSkillsRequest>,
) -> Response {
    let services_path = match shared::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    match repositories::update_plugin_skills(&services_path, &plugin_id, &body.skills) {
        Ok(()) => {
            let p = pool.clone();
            let uid = user_ctx.user_id.clone();
            let pid = plugin_id.clone();
            tokio::spawn(async move {
                activity::record(
                    &p,
                    NewActivity::entity_updated(&uid, ActivityEntity::Plugin, &pid, &pid),
                )
                .await;
            });
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, plugin_id = %plugin_id, "Failed to update plugin skills");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}
