use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::types::{
    CreateUserHookRequest, CreateUserMcpServerRequest, ForkAgentRequest, ForkHookRequest,
    ForkMcpServerRequest, ForkSkillRequest, UserContext,
};
use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;
use systemprompt::models::ProfileBootstrap;

use super::fs_readers::{read_agent_from_fs, read_hook_from_fs, read_mcp_server_from_fs};

#[allow(clippy::result_large_err)]
pub(crate) fn get_services_path() -> Result<std::path::PathBuf, Response> {
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

pub(crate) async fn fork_org_skill_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<ForkSkillRequest>,
) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return r,
    };

    let skill_dir = services_path.join("skills").join(&req.org_skill_id);
    if !skill_dir.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Org skill not found" })),
        )
            .into_response();
    }

    let config_path = skill_dir.join("config.yaml");
    let (name, description, tags) = if config_path.exists() {
        let cfg_text = std::fs::read_to_string(&config_path).unwrap_or_default();
        let cfg: serde_yaml::Value =
            serde_yaml::from_str(&cfg_text).unwrap_or(serde_yaml::Value::Null);
        let name = cfg
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(&req.org_skill_id)
            .to_string();
        let desc = cfg
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let tags: Vec<String> = cfg
            .get("tags")
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        (name, desc, tags)
    } else {
        (req.org_skill_id.clone(), String::new(), vec![])
    };

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

    let skill_id = req.skill_id.unwrap_or_else(|| req.org_skill_id.clone());

    let create_req = crate::admin::types::CreateSkillRequest {
        skill_id,
        name,
        description,
        content,
        tags,
        base_skill_id: Some(req.org_skill_id),
    };

    match repositories::create_user_skill(&pool, &user_ctx.user_id, &create_req).await {
        Ok(skill) => {
            let _ = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await;
            (StatusCode::CREATED, Json(json!(skill))).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to fork org skill");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to fork skill" })),
            )
                .into_response()
        }
    }
}

pub(crate) async fn fork_org_agent_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<ForkAgentRequest>,
) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return r,
    };

    let agents_path = services_path.join("agents");
    let (name, description, system_prompt) = read_agent_from_fs(&agents_path, &req.org_agent_id);

    if name.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Org agent not found" })),
        )
            .into_response();
    }

    let agent_id = req.agent_id.unwrap_or_else(|| req.org_agent_id.clone());

    let create_req = crate::admin::types::CreateUserAgentRequest {
        agent_id,
        name,
        description,
        system_prompt,
        base_agent_id: Some(req.org_agent_id),
    };

    match repositories::create_user_agent(&pool, &user_ctx.user_id, &create_req).await {
        Ok(agent) => {
            let _ = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await;
            (StatusCode::CREATED, Json(json!(agent))).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to fork org agent");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to fork agent" })),
            )
                .into_response()
        }
    }
}

pub(crate) async fn fork_org_mcp_server_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<ForkMcpServerRequest>,
) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return r,
    };

    let mcp = read_mcp_server_from_fs(&services_path, &req.org_mcp_server_id);

    let server_id = req
        .mcp_server_id
        .unwrap_or_else(|| req.org_mcp_server_id.clone());

    let create_req = CreateUserMcpServerRequest {
        mcp_server_id: server_id,
        name: mcp.name,
        description: mcp.description,
        binary: mcp.binary,
        package_name: mcp.package_name,
        port: mcp.port,
        endpoint: mcp.endpoint,
        oauth_required: mcp.oauth_required,
        oauth_scopes: mcp.oauth_scopes,
        oauth_audience: mcp.oauth_audience,
        base_mcp_server_id: Some(req.org_mcp_server_id),
    };

    match repositories::create_user_mcp_server(&pool, &user_ctx.user_id, &create_req).await {
        Ok(server) => {
            let _ = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await;
            (StatusCode::CREATED, Json(json!(server))).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to fork org MCP server");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to fork MCP server" })),
            )
                .into_response()
        }
    }
}

pub(crate) async fn fork_org_hook_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<ForkHookRequest>,
) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return r,
    };

    let hook = read_hook_from_fs(&services_path, &req.org_hook_id);
    let Some(hook) = hook else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Org hook not found" })),
        )
            .into_response();
    };

    let hook_id = req.hook_id.unwrap_or_else(|| req.org_hook_id.clone());

    let create_req = CreateUserHookRequest {
        hook_id,
        name: hook.name,
        description: hook.description,
        event: hook.event,
        matcher: hook.matcher,
        command: hook.command,
        is_async: hook.is_async,
        base_hook_id: Some(req.org_hook_id),
    };

    match repositories::create_user_hook(&pool, &user_ctx.user_id, &create_req).await {
        Ok(hook) => {
            let _ = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await;
            (StatusCode::CREATED, Json(json!(hook))).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to fork org hook");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to fork hook" })),
            )
                .into_response()
        }
    }
}
