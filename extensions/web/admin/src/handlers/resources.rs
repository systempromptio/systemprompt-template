use std::sync::Arc;

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;

use systemprompt::identifiers::AgentId;

use crate::activity::{self, ActivityEntity, NewActivity};
use crate::handlers::shared;
use crate::repositories;
use crate::types::{
    CreateAgentRequest, CreateUserAgentRequest, UpdateAgentRequest, UserContext,
};

use super::responses::AgentsListResponse;

pub fn get_services_path() -> Result<std::path::PathBuf, Box<Response>> {
    shared::get_services_path()
}

pub async fn list_agents_handler(Extension(user_ctx): Extension<UserContext>) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    let agents = match repositories::list_agents(&services_path) {
        Ok(a) => a,
        Err(e) => {
            tracing::error!(error = %e, "Failed to list agents");
            return shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
        }
    };
    if user_ctx.is_admin {
        return Json(AgentsListResponse { agents }).into_response();
    }
    let plugins = repositories::list_plugins_for_roles(&services_path, &user_ctx.roles)
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list plugins for role filtering");
            Vec::new()
        });
    let visible_ids: std::collections::HashSet<String> = plugins
        .iter()
        .flat_map(|p| p.agents.iter().map(|a| a.id.clone()))
        .collect();
    let filtered: Vec<_> = agents
        .into_iter()
        .filter(|a| visible_ids.contains(a.id.as_str()))
        .collect();
    Json(AgentsListResponse { agents: filtered }).into_response()
}

pub async fn get_agent_handler(Path(agent_id): Path<String>) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match repositories::find_agent(&services_path, &agent_id) {
        Ok(Some(agent)) => Json(agent).into_response(),
        Ok(None) => shared::error_response(StatusCode::NOT_FOUND, "Agent not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get agent");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

pub async fn create_agent_handler(
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
            let uid = user_ctx.user_id;
            tokio::spawn(async move {
                activity::record(
                    &pool,
                    NewActivity::entity_created(&uid, ActivityEntity::Agent, aid.as_str(), &name),
                )
                .await;
            });
            (StatusCode::CREATED, Json(agent)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create agent");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

pub async fn update_agent_handler(
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
            let uid = user_ctx.user_id;
            tokio::spawn(async move {
                activity::record(
                    &pool,
                    NewActivity::entity_updated(&uid, ActivityEntity::Agent, &aid, &name),
                )
                .await;
            });
            Json(agent).into_response()
        }
        Ok(None) => shared::error_response(StatusCode::NOT_FOUND, "Agent not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to update agent");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

pub async fn delete_agent_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(agent_id): Path<String>,
) -> Response {
    if !user_ctx.is_admin {
        return shared::error_response(StatusCode::FORBIDDEN, "Admin access required");
    }
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match repositories::delete_agent(&services_path, &agent_id) {
        Ok(true) => {
            let aid = agent_id.clone();
            let uid = user_ctx.user_id;
            tokio::spawn(async move {
                activity::record(
                    &pool,
                    NewActivity::entity_deleted(&uid, ActivityEntity::Agent, &aid, &aid),
                )
                .await;
            });
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(false) => shared::error_response(StatusCode::NOT_FOUND, "Agent not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to delete agent");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

pub async fn create_user_agent_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Json(body): Json<CreateUserAgentRequest>,
) -> Response {
    match repositories::create_user_agent(&pool, &user_ctx.user_id, &body).await {
        Ok(agent) => {
            let name = agent.name.clone();
            let aid = agent.agent_id.clone();
            let uid = user_ctx.user_id;
            tokio::spawn(async move {
                activity::record(
                    &pool,
                    NewActivity::entity_created(
                        &uid,
                        ActivityEntity::UserAgent,
                        aid.as_str(),
                        &name,
                    ),
                )
                .await;
            });
            (StatusCode::CREATED, Json(agent)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create user agent");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

pub async fn delete_user_agent_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(agent_id_raw): Path<String>,
) -> Response {
    let agent_id = AgentId::new(agent_id_raw);
    match repositories::delete_user_agent(&pool, &user_ctx.user_id, &agent_id).await {
        Ok(true) => {
            let aid = agent_id.clone();
            let uid = user_ctx.user_id;
            tokio::spawn(async move {
                activity::record(
                    &pool,
                    NewActivity::entity_deleted(
                        &uid,
                        ActivityEntity::UserAgent,
                        aid.as_str(),
                        aid.as_str(),
                    ),
                )
                .await;
            });
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(false) => shared::error_response(StatusCode::NOT_FOUND, "User agent not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to delete user agent");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}
