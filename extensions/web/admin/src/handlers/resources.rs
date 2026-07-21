//! HTTP handlers for managed resource listings.

use axum::Json;
use axum::extract::{Extension, Path};
use axum::response::{IntoResponse, Response};

use systemprompt::identifiers::AgentId;

use crate::error::{AdminError, AdminResult};
use crate::handlers::shared;
use crate::repositories;
use crate::types::UserContext;

use super::responses::AgentsListResponse;

pub(crate) async fn list_agents_handler(
    Extension(user_ctx): Extension<UserContext>,
) -> AdminResult<Response> {
    let services_path = shared::get_services_path()?;
    let agents =
        repositories::config::agents::list_agents(&services_path).map_err(AdminError::internal)?;
    if user_ctx.is_admin {
        return Ok(Json(AgentsListResponse { agents }).into_response());
    }
    let plugins =
        repositories::marketplace::plugins::list_plugins_for_roles(&services_path, &user_ctx.roles)
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
    Ok(Json(AgentsListResponse { agents: filtered }).into_response())
}

pub(crate) async fn get_agent_handler(Path(agent_id): Path<String>) -> AdminResult<Response> {
    let services_path = shared::get_services_path()?;
    let agent = repositories::config::agents::find_agent(&services_path, &AgentId::new(agent_id))
        .map_err(AdminError::internal)?
        .ok_or_else(|| AdminError::NotFound("Agent not found".to_owned()))?;
    Ok(Json(agent).into_response())
}
