use std::collections::HashSet;
use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::types::UserContext;
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use sqlx::PgPool;
use systemprompt::models::ProfileBootstrap;

#[allow(clippy::result_large_err)]
fn get_services_path() -> Result<std::path::PathBuf, Response> {
    ProfileBootstrap::get()
        .map(|p| std::path::PathBuf::from(&p.paths.services))
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to get profile bootstrap");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to load profile"})),
            )
                .into_response()
        })
}

pub(crate) async fn list_forkable_plugins_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return r,
    };

    let org_plugins = repositories::list_plugins_for_roles(&services_path, &user_ctx.roles)
        .unwrap_or_default();

    let user_plugins = repositories::list_user_plugins(&pool, &user_ctx.user_id)
        .await
        .unwrap_or_default();

    let forked_base_ids: HashSet<String> = user_plugins
        .iter()
        .filter_map(|p| p.base_plugin_id.clone())
        .collect();

    let items: Vec<serde_json::Value> = org_plugins
        .iter()
        .map(|p| {
            json!({
                "id": p.id,
                "name": p.name,
                "description": p.description,
                "skill_count": p.skills.len(),
                "agent_count": p.agents.len(),
                "mcp_count": p.mcp_servers.len(),
                "hook_count": p.hooks.len(),
                "already_forked": forked_base_ids.contains(&p.id),
            })
        })
        .collect();

    Json(json!({ "plugins": items })).into_response()
}

pub(crate) async fn list_forkable_skills_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return r,
    };

    let org_plugins = repositories::list_plugins_for_roles(&services_path, &user_ctx.roles)
        .unwrap_or_default();

    let user_skills = repositories::list_user_skills(&pool, &user_ctx.user_id)
        .await
        .unwrap_or_default();

    let forked_base_ids: HashSet<String> = user_skills
        .iter()
        .filter_map(|s| s.base_skill_id.clone())
        .collect();

    let mut seen = HashSet::new();
    let mut items = Vec::new();
    for plugin in &org_plugins {
        for skill in &plugin.skills {
            if seen.insert(skill.id.clone()) {
                items.push(json!({
                    "id": skill.id,
                    "name": skill.name,
                    "description": skill.description,
                    "plugin_id": plugin.id,
                    "plugin_name": plugin.name,
                    "already_forked": forked_base_ids.contains(&skill.id),
                }));
            }
        }
    }

    Json(json!({ "skills": items })).into_response()
}

pub(crate) async fn list_forkable_agents_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return r,
    };

    let org_plugins = repositories::list_plugins_for_roles(&services_path, &user_ctx.roles)
        .unwrap_or_default();

    let user_agents = repositories::list_user_agents(&pool, &user_ctx.user_id)
        .await
        .unwrap_or_default();

    let forked_base_ids: HashSet<String> = user_agents
        .iter()
        .filter_map(|a| a.base_agent_id.clone())
        .collect();

    let mut seen = HashSet::new();
    let mut items = Vec::new();
    for plugin in &org_plugins {
        for agent in &plugin.agents {
            if seen.insert(agent.id.clone()) {
                items.push(json!({
                    "id": agent.id,
                    "name": agent.name,
                    "description": agent.description,
                    "plugin_id": plugin.id,
                    "plugin_name": plugin.name,
                    "already_forked": forked_base_ids.contains(&agent.id),
                }));
            }
        }
    }

    Json(json!({ "agents": items })).into_response()
}

pub(crate) async fn list_forkable_mcp_servers_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return r,
    };

    let org_plugins = repositories::list_plugins_for_roles(&services_path, &user_ctx.roles)
        .unwrap_or_default();

    let user_mcp = repositories::list_user_mcp_servers(&pool, &user_ctx.user_id)
        .await
        .unwrap_or_default();

    let forked_base_ids: HashSet<String> = user_mcp
        .iter()
        .filter_map(|m| m.base_mcp_server_id.clone())
        .collect();

    let mut seen = HashSet::new();
    let mut items = Vec::new();
    for plugin in &org_plugins {
        for mcp_id in &plugin.mcp_servers {
            if seen.insert(mcp_id.clone()) {
                items.push(json!({
                    "id": mcp_id,
                    "name": mcp_id,
                    "plugin_id": plugin.id,
                    "plugin_name": plugin.name,
                    "already_forked": forked_base_ids.contains(mcp_id),
                }));
            }
        }
    }

    Json(json!({ "mcp_servers": items })).into_response()
}

pub(crate) async fn list_forkable_hooks_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return r,
    };

    let org_plugins = repositories::list_plugins_for_roles(&services_path, &user_ctx.roles)
        .unwrap_or_default();

    let user_hooks = repositories::list_user_hooks(&pool, &user_ctx.user_id)
        .await
        .unwrap_or_default();

    let forked_base_ids: HashSet<String> = user_hooks
        .iter()
        .filter_map(|h| h.base_hook_id.clone())
        .collect();

    let mut seen = HashSet::new();
    let mut items = Vec::new();
    for plugin in &org_plugins {
        for hook in &plugin.hooks {
            if seen.insert(hook.id.clone()) {
                items.push(json!({
                    "id": hook.id,
                    "name": hook.name,
                    "event": hook.event,
                    "plugin_id": plugin.id,
                    "plugin_name": plugin.name,
                    "already_forked": forked_base_ids.contains(&hook.id),
                }));
            }
        }
    }

    Json(json!({ "hooks": items })).into_response()
}
