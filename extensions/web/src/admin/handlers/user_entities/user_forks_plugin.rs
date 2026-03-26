use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::types::{
    CreateUserHookRequest, CreateUserMcpServerRequest, CreateUserPluginRequest, ForkPluginRequest,
    UserContext,
};
use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

use super::fs_readers::{read_agent_from_fs, read_mcp_server_from_fs};
use super::user_forks::get_services_path;

#[allow(clippy::too_many_lines)]
pub(crate) async fn fork_org_plugin_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<ForkPluginRequest>,
) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return r,
    };

    let org_plugins = repositories::list_plugins_for_roles(&services_path, &user_ctx.roles)
        .unwrap_or_default();

    let org_plugin = org_plugins.iter().find(|p| p.id == req.org_plugin_id);
    let Some(org_plugin) = org_plugin else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Org plugin not found or not accessible" })),
        )
            .into_response();
    };

    let plugin_id = req
        .plugin_id
        .unwrap_or_else(|| req.org_plugin_id.clone());

    let create_plugin_req = CreateUserPluginRequest {
        plugin_id: plugin_id.clone(),
        name: org_plugin.name.clone(),
        description: org_plugin.description.clone(),
        version: "1.0.0".to_string(),
        category: String::new(),
        keywords: vec![],
        author_name: user_ctx.username.clone(),
        base_plugin_id: Some(req.org_plugin_id.clone()),
    };

    let plugin = match repositories::create_user_plugin(
        &pool,
        &user_ctx.user_id,
        &create_plugin_req,
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(error = %e, "Failed to create forked plugin");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to fork plugin" })),
            )
                .into_response();
        }
    };

    let mut forked_skill_ids = Vec::new();
    for skill_info in &org_plugin.skills {
        let skill_dir = services_path.join("skills").join(&skill_info.id);
        let content = {
            let skill_md = skill_dir.join("SKILL.md");
            let index_md = skill_dir.join("index.md");
            if skill_md.exists() {
                std::fs::read_to_string(&skill_md).unwrap_or_default()
            } else if index_md.exists() {
                std::fs::read_to_string(&index_md).unwrap_or_default()
            } else {
                String::new()
            }
        };
        let create_req = crate::admin::types::CreateSkillRequest {
            skill_id: skill_info.id.clone(),
            name: skill_info.name.clone(),
            description: skill_info.description.clone(),
            content,
            tags: vec![],
            base_skill_id: Some(skill_info.id.clone()),
        };
        match repositories::create_user_skill(&pool, &user_ctx.user_id, &create_req).await {
            Ok(s) => forked_skill_ids.push(s.id),
            Err(e) => tracing::warn!(
                error = %e,
                skill = %skill_info.id,
                "Failed to fork skill during plugin fork"
            ),
        }
    }

    let agents_path = services_path.join("agents");
    let mut forked_agent_ids = Vec::new();
    for agent_info in &org_plugin.agents {
        let (name, description, system_prompt) =
            read_agent_from_fs(&agents_path, &agent_info.id);
        let create_req = crate::admin::types::CreateUserAgentRequest {
            agent_id: agent_info.id.clone(),
            name: if name.is_empty() {
                agent_info.name.clone()
            } else {
                name
            },
            description: if description.is_empty() {
                agent_info.description.clone()
            } else {
                description
            },
            system_prompt,
            base_agent_id: Some(agent_info.id.clone()),
        };
        match repositories::create_user_agent(&pool, &user_ctx.user_id, &create_req).await {
            Ok(a) => forked_agent_ids.push(a.id),
            Err(e) => tracing::warn!(
                error = %e,
                agent = %agent_info.id,
                "Failed to fork agent during plugin fork"
            ),
        }
    }

    let mut forked_mcp_ids = Vec::new();
    for mcp_id in &org_plugin.mcp_servers {
        let mcp = read_mcp_server_from_fs(&services_path, mcp_id);
        let create_req = CreateUserMcpServerRequest {
            mcp_server_id: mcp_id.clone(),
            name: mcp.name,
            description: mcp.description,
            binary: mcp.binary,
            package_name: mcp.package_name,
            port: mcp.port,
            endpoint: mcp.endpoint,
            oauth_required: mcp.oauth_required,
            oauth_scopes: mcp.oauth_scopes,
            oauth_audience: mcp.oauth_audience,
            base_mcp_server_id: Some(mcp_id.clone()),
        };
        match repositories::create_user_mcp_server(&pool, &user_ctx.user_id, &create_req)
            .await
        {
            Ok(m) => forked_mcp_ids.push(m.id),
            Err(e) => tracing::warn!(
                error = %e,
                mcp = %mcp_id,
                "Failed to fork MCP server during plugin fork"
            ),
        }
    }

    let mut forked_hook_ids = Vec::new();
    for hook_info in &org_plugin.hooks {
        let create_req = CreateUserHookRequest {
            hook_id: hook_info.id.clone(),
            name: hook_info.name.clone(),
            description: hook_info.description.clone(),
            event: hook_info.event.clone(),
            matcher: hook_info.matcher.clone(),
            command: hook_info.command.clone(),
            is_async: hook_info.is_async,
            base_hook_id: Some(hook_info.id.clone()),
        };
        match repositories::create_user_hook(&pool, &user_ctx.user_id, &create_req).await {
            Ok(h) => forked_hook_ids.push(h.id),
            Err(e) => tracing::warn!(
                error = %e,
                hook = %hook_info.id,
                "Failed to fork hook during plugin fork"
            ),
        }
    }

    let _ = repositories::set_plugin_skills(&pool, &plugin.id, &forked_skill_ids).await;
    let _ = repositories::set_plugin_agents(&pool, &plugin.id, &forked_agent_ids).await;
    let _ = repositories::set_plugin_mcp_servers(&pool, &plugin.id, &forked_mcp_ids).await;
    let _ = repositories::set_plugin_hooks(&pool, &plugin.id, &forked_hook_ids).await;
    let _ = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await;

    (
        StatusCode::CREATED,
        Json(json!({
            "plugin": plugin,
            "forked_skills": forked_skill_ids.len(),
            "forked_agents": forked_agent_ids.len(),
            "forked_mcp_servers": forked_mcp_ids.len(),
            "forked_hooks": forked_hook_ids.len(),
        })),
    )
        .into_response()
}
