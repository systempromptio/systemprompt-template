use std::collections::HashSet;
use std::sync::Arc;

use crate::handlers::shared;
use crate::repositories;
use crate::types::UserContext;
use axum::{
    extract::{Extension, State},
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;

use crate::handlers::responses::{
    AgentsListResponse, ForkableAgentItem, ForkablePluginItem, ForkableSkillItem,
    PluginsListResponse, SkillsListResponse,
};

fn get_services_path() -> Result<std::path::PathBuf, Box<Response>> {
    shared::get_services_path()
}

pub async fn list_forkable_plugins_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let org_plugins = repositories::list_plugins_for_roles(&services_path, &user_ctx.roles)
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list org plugins for fork listing");
            Vec::new()
        });

    let user_plugins = repositories::list_user_plugins(&pool, &user_ctx.user_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list user plugins for fork listing");
            Vec::new()
        });

    let forked_base_ids: HashSet<String> = user_plugins
        .into_iter()
        .filter_map(|p| p.base_plugin_id)
        .collect();

    let items: Vec<ForkablePluginItem> = org_plugins
        .into_iter()
        .filter(|p| !p.id.is_empty())
        .map(|p| ForkablePluginItem {
            already_forked: forked_base_ids.contains(&p.id),
            skill_count: p.skills.len(),
            agent_count: p.agents.len(),
            mcp_count: p.mcp_servers.len(),
            id: p.id,
            name: p.name,
            description: p.description,
        })
        .collect();

    Json(PluginsListResponse { plugins: items }).into_response()
}

pub async fn list_forkable_skills_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let org_plugins = repositories::list_plugins_for_roles(&services_path, &user_ctx.roles)
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list org plugins for skill fork listing");
            Vec::new()
        });

    let user_skills = repositories::list_user_skills(&pool, &user_ctx.user_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list user skills for fork listing");
            Vec::new()
        });

    let forked_base_ids: HashSet<String> = user_skills
        .into_iter()
        .filter_map(|s| s.base_skill_id.map(|id| id.to_string()))
        .collect();

    let mut seen: HashSet<String> = HashSet::new();
    let mut items = Vec::new();
    for plugin in org_plugins {
        for skill in plugin.skills {
            let skill_id_str = skill.id.to_string();
            if !seen.insert(skill_id_str.clone()) {
                continue;
            }
            let already_forked = forked_base_ids.contains(&skill_id_str);
            items.push(ForkableSkillItem {
                id: skill_id_str,
                name: skill.name,
                description: skill.description,
                plugin_id: plugin.id.clone(),
                plugin_name: plugin.name.clone(),
                already_forked,
            });
        }
    }

    Json(SkillsListResponse { skills: items }).into_response()
}

pub async fn list_forkable_agents_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let org_plugins = repositories::list_plugins_for_roles(&services_path, &user_ctx.roles)
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list org plugins for agent fork listing");
            Vec::new()
        });

    let user_agents = repositories::list_user_agents(&pool, &user_ctx.user_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list user agents for fork listing");
            Vec::new()
        });

    let forked_base_ids: HashSet<String> = user_agents
        .into_iter()
        .filter_map(|a| a.base_agent_id.map(|id| id.to_string()))
        .collect();

    let mut seen: HashSet<String> = HashSet::new();
    let mut items = Vec::new();
    for plugin in org_plugins {
        for agent in plugin.agents {
            let agent_id_str = agent.id.to_string();
            if !seen.insert(agent_id_str.clone()) {
                continue;
            }
            let already_forked = forked_base_ids.contains(&agent_id_str);
            items.push(ForkableAgentItem {
                id: agent_id_str,
                name: agent.name,
                description: agent.description,
                plugin_id: plugin.id.clone(),
                plugin_name: plugin.name.clone(),
                already_forked,
            });
        }
    }

    Json(AgentsListResponse { agents: items }).into_response()
}
