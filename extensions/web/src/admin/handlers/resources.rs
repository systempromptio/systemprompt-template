use std::sync::Arc;

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;
use systemprompt::models::ProfileBootstrap;

use crate::admin::activity::{self, ActivityEntity, NewActivity};
use crate::admin::repositories;
use crate::admin::types::{
    CreateAgentRequest, CreateUserAgentRequest, UpdateAgentRequest, UserContext,
};

pub(crate) fn get_services_path() -> Result<std::path::PathBuf, Box<Response>> {
    ProfileBootstrap::get()
        .map(|p| std::path::PathBuf::from(&p.paths.services))
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to get profile bootstrap");
            Box::new(
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Failed to load profile"})),
                )
                    .into_response(),
            )
        })
}

pub(crate) async fn list_agents_handler(Extension(user_ctx): Extension<UserContext>) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    let agents = match repositories::list_agents(&services_path) {
        Ok(a) => a,
        Err(e) => {
            tracing::error!(error = %e, "Failed to list agents");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };
    if user_ctx.is_admin {
        return Json(agents).into_response();
    }
    let plugins =
        repositories::list_plugins_for_roles(&services_path, &user_ctx.roles).unwrap_or_default();
    let visible_ids: std::collections::HashSet<String> = plugins
        .iter()
        .flat_map(|p| p.agents.iter().map(|a| a.id.clone()))
        .collect();
    let filtered: Vec<_> = agents
        .into_iter()
        .filter(|a| visible_ids.contains(&a.id))
        .collect();
    Json(filtered).into_response()
}

pub(crate) async fn get_agent_handler(Path(agent_id): Path<String>) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match repositories::get_agent(&services_path, &agent_id) {
        Ok(Some(agent)) => Json(agent).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Agent not found"})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get agent");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn create_agent_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Json(body): Json<CreateAgentRequest>,
) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match repositories::create_agent(&services_path, &body) {
        Ok(agent) => {
            let name = agent.name.clone();
            let aid = agent.id.clone();
            let uid = user_ctx.user_id.clone();
            tokio::spawn(async move {
                activity::record(
                    &pool,
                    NewActivity::entity_created(&uid, ActivityEntity::Agent, &aid, &name),
                )
                .await;
            });
            (StatusCode::CREATED, Json(agent)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create agent");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn update_agent_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(agent_id): Path<String>,
    Json(body): Json<UpdateAgentRequest>,
) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match repositories::update_agent(&services_path, &agent_id, &body) {
        Ok(Some(agent)) => {
            let name = agent.name.clone();
            let aid = agent_id.clone();
            let uid = user_ctx.user_id.clone();
            tokio::spawn(async move {
                activity::record(
                    &pool,
                    NewActivity::entity_updated(&uid, ActivityEntity::Agent, &aid, &name),
                )
                .await;
            });
            Json(agent).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Agent not found"})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to update agent");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn delete_agent_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(agent_id): Path<String>,
) -> Response {
    if !user_ctx.is_admin {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        )
            .into_response();
    }
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match repositories::delete_agent(&services_path, &agent_id) {
        Ok(true) => {
            let aid = agent_id.clone();
            let uid = user_ctx.user_id.clone();
            tokio::spawn(async move {
                activity::record(
                    &pool,
                    NewActivity::entity_deleted(&uid, ActivityEntity::Agent, &aid, &aid),
                )
                .await;
            });
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Agent not found"})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to delete agent");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn create_user_agent_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Json(body): Json<CreateUserAgentRequest>,
) -> Response {
    match repositories::create_user_agent(&pool, &user_ctx.user_id, &body).await {
        Ok(agent) => {
            let name = agent.name.clone();
            let aid = agent.agent_id.clone();
            let uid = user_ctx.user_id.clone();
            tokio::spawn(async move {
                activity::record(
                    &pool,
                    NewActivity::entity_created(&uid, ActivityEntity::UserAgent, &aid, &name),
                )
                .await;
            });
            (StatusCode::CREATED, Json(agent)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create user agent");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn delete_user_agent_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(agent_id): Path<String>,
) -> Response {
    match repositories::delete_user_agent(&pool, &user_ctx.user_id, &agent_id).await {
        Ok(true) => {
            let aid = agent_id.clone();
            let uid = user_ctx.user_id.clone();
            tokio::spawn(async move {
                activity::record(
                    &pool,
                    NewActivity::entity_deleted(&uid, ActivityEntity::UserAgent, &aid, &aid),
                )
                .await;
            });
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "User agent not found"})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to delete user agent");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}
