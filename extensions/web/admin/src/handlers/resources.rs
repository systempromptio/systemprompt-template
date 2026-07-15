use axum::Json;
use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use systemprompt::identifiers::AgentId;

use crate::handlers::shared;
use crate::repositories;
use crate::types::UserContext;

use super::responses::AgentsListResponse;

pub(crate) fn get_services_path() -> Result<std::path::PathBuf, Box<Response>> {
    shared::get_services_path()
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
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
            );
        },
    };
    if user_ctx.is_admin {
        return Json(AgentsListResponse { agents }).into_response();
    }
    let plugins = repositories::list_plugins_for_roles(&services_path, &user_ctx.roles)
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list plugins for role filtering");
            Vec::new()
        });
    let visible_ids: std::collections::HashSet<AgentId> = plugins
        .iter()
        .flat_map(|p| p.agents.iter().map(|a| a.id.clone()))
        .collect();
    let filtered: Vec<_> = agents
        .into_iter()
        .filter(|a| visible_ids.contains(&a.id))
        .collect();
    Json(AgentsListResponse { agents: filtered }).into_response()
}

pub(crate) async fn get_agent_handler(Path(agent_id): Path<String>) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match repositories::find_agent(&services_path, &AgentId::new(agent_id)) {
        Ok(Some(agent)) => Json(agent).into_response(),
        Ok(None) => shared::error_response(StatusCode::NOT_FOUND, "Agent not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get agent");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
        },
    }
}
