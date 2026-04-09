use std::sync::Arc;

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;
use systemprompt::models::ProfileBootstrap;

use systemprompt::identifiers::{SkillId, UserId};

use crate::admin::handlers::shared;
use crate::admin::repositories;
use crate::admin::types::{CreateSkillRequest, UserContext, UserQuery};

use super::responses::{PluginsListResponse, SkillsListResponse};

pub(crate) async fn list_plugins_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<UserQuery>,
) -> Response {
    let services_path = match shared::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let mut plugins = match repositories::list_plugins_for_roles(&services_path, &user_ctx.roles) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(error = %e, "Failed to list plugins");
            return shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
        }
    };

    let default_user_id = UserId::new("admin");
    let user_id = query
        .user_id
        .as_ref()
        .map_or_else(|| default_user_id.clone(), UserId::new);
    if let Ok(custom_skills) = repositories::list_user_skills(&pool, &user_id).await {
        if !custom_skills.is_empty() {
            let skill_infos: Vec<crate::admin::types::SkillInfo> = custom_skills
                .into_iter()
                .map(|s| crate::admin::types::SkillInfo {
                    id: s.skill_id.to_string(),
                    name: s.name,
                    description: s.description,
                    command: format!("/custom:{}", s.skill_id),
                    source: "custom".to_string(),
                    enabled: s.enabled,
                    required_secrets: Vec::new(),
                })
                .collect();

            plugins.push(crate::admin::types::PluginOverview {
                id: "custom".to_string(),
                name: "Custom Skills".to_string(),
                description: "User-created custom skills".to_string(),
                enabled: true,
                skills: skill_infos,
                agents: vec![],
                mcp_servers: vec![],
                hooks: vec![],
                depends: vec![],
            });
        }
    }

    Json(PluginsListResponse { plugins }).into_response()
}

pub(crate) async fn list_skills_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let skills = match repositories::list_agent_skills(&pool).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = %e, "Failed to list skills");
            return shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
        }
    };
    if user_ctx.is_admin {
        return Json(SkillsListResponse { skills }).into_response();
    }
    let services_path = match ProfileBootstrap::get() {
        Ok(profile) => std::path::PathBuf::from(&profile.paths.services),
        Err(_) => return Json(SkillsListResponse { skills }).into_response(),
    };
    let plugins = repositories::list_plugins_for_roles(&services_path, &user_ctx.roles)
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list plugins for role filtering");
            Vec::new()
        });
    let visible_ids: std::collections::HashSet<String> = plugins
        .iter()
        .flat_map(|p| p.skills.iter().map(|s| s.id.as_str().to_string()))
        .collect();
    let filtered: Vec<_> = skills
        .into_iter()
        .filter(|s| visible_ids.contains(s.skill_id.as_str()))
        .collect();
    Json(SkillsListResponse { skills: filtered }).into_response()
}

pub(crate) async fn get_skill_handler(
    State(pool): State<Arc<PgPool>>,
    Path(skill_id_raw): Path<String>,
) -> Response {
    let skill_id = SkillId::new(skill_id_raw);
    match repositories::find_agent_skill(&pool, &skill_id).await {
        Ok(Some(skill)) => Json(skill).into_response(),
        Ok(None) => shared::error_response(StatusCode::NOT_FOUND, "Skill not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get skill");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

pub(crate) async fn create_skill_handler(
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<UserQuery>,
    Json(body): Json<CreateSkillRequest>,
) -> Response {
    let default_user_id = UserId::new("admin");
    let user_id = query
        .user_id
        .as_ref()
        .map_or_else(|| default_user_id.clone(), UserId::new);
    match repositories::create_user_skill(&pool, &user_id, &body).await {
        Ok(skill) => {
            let name = body.name.clone();
            let skill_id_clone = skill.skill_id.clone();
            let uid = user_id.clone();
            let p = pool.clone();
            tokio::spawn(async move {
                crate::admin::activity::record(
                    &p,
                    crate::admin::activity::NewActivity::entity_created(
                        &uid,
                        crate::admin::activity::ActivityEntity::UserSkill,
                        skill_id_clone.as_str(),
                        &name,
                    ),
                )
                .await;
            });
            (StatusCode::CREATED, Json(skill)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create skill");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}
