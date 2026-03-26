use std::sync::Arc;

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;
use systemprompt::models::ProfileBootstrap;

use crate::admin::repositories;
use crate::admin::types::{CreateSkillRequest, UserContext, UserQuery};

pub(crate) async fn list_plugins_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<UserQuery>,
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

    let mut plugins = match repositories::list_plugins_for_roles(&services_path, &user_ctx.roles) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(error = %e, "Failed to list plugins");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };

    let user_id = query.user_id.as_deref().unwrap_or("admin");
    if let Ok(custom_skills) = repositories::list_user_skills(&pool, user_id).await {
        if !custom_skills.is_empty() {
            let skill_infos: Vec<crate::admin::types::SkillInfo> = custom_skills
                .into_iter()
                .map(|s| crate::admin::types::SkillInfo {
                    id: s.skill_id.clone(),
                    name: s.name,
                    description: s.description,
                    command: format!("/custom:{}", s.skill_id),
                    source: "custom".to_string(),
                    enabled: s.enabled,
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

    Json(plugins).into_response()
}

pub(crate) async fn list_skills_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let skills = match repositories::list_agent_skills(&pool).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = %e, "Failed to list skills");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };
    if user_ctx.is_admin {
        return Json(skills).into_response();
    }
    let services_path = match ProfileBootstrap::get() {
        Ok(profile) => std::path::PathBuf::from(&profile.paths.services),
        Err(_) => return Json(skills).into_response(),
    };
    let plugins =
        repositories::list_plugins_for_roles(&services_path, &user_ctx.roles).unwrap_or_default();
    let visible_ids: std::collections::HashSet<String> = plugins
        .iter()
        .flat_map(|p| p.skills.iter().map(|s| s.id.clone()))
        .collect();
    let filtered: Vec<_> = skills
        .into_iter()
        .filter(|s| visible_ids.contains(&s.skill_id))
        .collect();
    Json(filtered).into_response()
}

pub(crate) async fn get_skill_handler(
    State(pool): State<Arc<PgPool>>,
    Path(skill_id): Path<String>,
) -> Response {
    match repositories::get_agent_skill(&pool, &skill_id).await {
        Ok(Some(skill)) => Json(skill).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Skill not found"})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get skill");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn create_skill_handler(
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<UserQuery>,
    Json(body): Json<CreateSkillRequest>,
) -> Response {
    let user_id = query.user_id.as_deref().unwrap_or("admin");
    match repositories::create_user_skill(&pool, user_id, &body).await {
        Ok(skill) => {
            let name = body.name.clone();
            let skill_id_clone = skill.skill_id.clone();
            let uid = user_id.to_string();
            let p = pool.clone();
            tokio::spawn(async move {
                crate::admin::activity::record(
                    &p,
                    crate::admin::activity::NewActivity::entity_created(
                        &uid,
                        crate::admin::activity::ActivityEntity::UserSkill,
                        &skill_id_clone,
                        &name,
                    ),
                )
                .await;
            });
            (StatusCode::CREATED, Json(skill)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create skill");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}
