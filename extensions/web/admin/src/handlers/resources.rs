//! HTTP handlers for managed resource listings.

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;
use std::sync::Arc;

use systemprompt::identifiers::AgentId;

use crate::error::{AdminError, AdminResult};
use crate::handlers::shared;
use crate::repositories;
use crate::types::UserContext;

use super::responses::AgentsListResponse;

pub(crate) async fn list_agents_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
) -> AdminResult<Response> {
    let services_path = shared::get_services_path()?;
    let agents = repositories::config::agents::list_configured_agents(&services_path)
        .map_err(AdminError::internal)?;
    // Why: `admin`, not `is_console`. The unfiltered list includes
    // `admin_console`, which a project manager is deliberately not granted;
    // they get the plugin-filtered view like every other role.
    if user_ctx.is_admin {
        return Ok(Json(AgentsListResponse { agents }).into_response());
    }
    let access = crate::authz::catalog::CatalogAccess::load(&pool, &user_ctx.user_id).await?;
    let ids = agents
        .iter()
        .filter(|a| a.enabled)
        .map(|a| a.id.to_string())
        .collect::<Vec<_>>();
    let allowed = access
        .allowed(systemprompt_security::authz::EntityKind::Agent, &ids)
        .await?;
    let filtered: Vec<_> = agents
        .into_iter()
        .filter(|a| a.enabled && allowed.contains(a.id.as_str()))
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
