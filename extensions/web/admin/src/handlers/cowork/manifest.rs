use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use systemprompt::models::AppPaths;
use systemprompt::security::manifest_signing;

use crate::handlers::shared;
use crate::repositories::user_agents::list_user_agents;
use crate::repositories::user_skills::list_user_skills;
use crate::repositories::users_grp::user_mcp_servers::list_user_mcp_servers;
use crate::repositories::users_grp::user_plugins::list_user_plugins;

use super::load_user_section;
use super::plugin_walker::{self, PluginEntry};
use super::types::{
    AgentEntry, ManagedMcpServer, Manifest, SkillEntry,
};

pub async fn handle(State(pool): State<Arc<PgPool>>, headers: HeaderMap) -> Response {
    let user_id = match super::validate_cowork_jwt(&headers) {
        Ok(id) => id,
        Err(r) => return *r,
    };

    let user = match load_user_section(&pool, &user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => return shared::error_response(StatusCode::NOT_FOUND, "User not found"),
        Err(e) => {
            tracing::error!(error = %e, "user lookup failed");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "User lookup failed",
            );
        },
    };

    let plugins_root = match AppPaths::get() {
        Ok(p) => p.system().services().join("plugins"),
        Err(e) => {
            tracing::error!(error = %e, "AppPaths::get failed");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Service paths unavailable",
            );
        },
    };

    let plugin_rows = match list_user_plugins(&pool, &user_id).await {
        Ok(rs) => rs,
        Err(e) => {
            tracing::error!(error = %e, "list_user_plugins failed");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Plugin listing failed",
            );
        },
    };
    let plugins: Vec<PluginEntry> = plugin_rows
        .iter()
        .filter(|p| p.enabled)
        .filter_map(|p| plugin_walker::build_entry(&plugins_root, &p.plugin_id, &p.version))
        .collect();

    let skills = match build_skills(&pool, &user_id).await {
        Ok(s) => s,
        Err(e) => return e,
    };
    let agents = match build_agents(&pool, &user_id).await {
        Ok(a) => a,
        Err(e) => return e,
    };
    let managed_mcp_servers = match build_mcp(&pool, &user_id).await {
        Ok(m) => m,
        Err(e) => return e,
    };

    let manifest_version = format!(
        "{}-{}",
        chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ"),
        short_hash(&plugins, &managed_mcp_servers, &skills, &agents),
    );
    let issued_at = chrono::Utc::now().to_rfc3339();
    let user_id_str = user_id.as_str().to_string();

    let mut manifest = Manifest {
        manifest_version,
        issued_at,
        user_id: user_id_str,
        tenant_id: None,
        user,
        plugins,
        skills,
        agents,
        managed_mcp_servers,
        revocations: Vec::new(),
        signature: None,
    };

    let canonical = match serde_json::to_string(&manifest) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = %e, "canonical serialise failed");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Manifest serialisation failed",
            );
        },
    };

    match manifest_signing::sign_payload(canonical.as_bytes()) {
        Ok(sig) => manifest.signature = Some(sig),
        Err(e) => {
            tracing::error!(error = %e, "manifest signing failed");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Manifest signing failed",
            );
        },
    }

    (StatusCode::OK, Json(manifest)).into_response()
}

async fn build_skills(
    pool: &PgPool,
    user_id: &systemprompt::identifiers::UserId,
) -> Result<Vec<SkillEntry>, Response> {
    let rows = list_user_skills(pool, user_id).await.map_err(|e| {
        tracing::error!(error = %e, "list_user_skills failed");
        shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Skill listing failed")
    })?;
    Ok(rows
        .into_iter()
        .filter(|s| s.enabled)
        .map(|s| {
            let mut h = Sha256::new();
            h.update(s.skill_id.as_str().as_bytes());
            h.update(b"\0");
            h.update(s.content.as_bytes());
            let sha256 = plugin_walker::hex_encode(&h.finalize());
            SkillEntry {
                id: s.skill_id.as_str().to_string(),
                name: s.name,
                description: s.description,
                file_path: String::new(),
                tags: s.tags,
                sha256,
                instructions: s.content,
            }
        })
        .collect())
}

async fn build_agents(
    pool: &PgPool,
    user_id: &systemprompt::identifiers::UserId,
) -> Result<Vec<AgentEntry>, Response> {
    let rows = list_user_agents(pool, user_id).await.map_err(|e| {
        tracing::error!(error = %e, "list_user_agents failed");
        shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Agent listing failed")
    })?;
    Ok(rows
        .into_iter()
        .filter(|a| a.enabled)
        .map(|a| AgentEntry {
            id: a.agent_id.as_str().to_string(),
            name: a.name.clone(),
            display_name: a.name,
            description: a.description,
            version: "1.0.0".into(),
            endpoint: format!("/api/v1/agents/{}", a.agent_id.as_str()),
            enabled: a.enabled,
            is_default: false,
            is_primary: false,
            provider: None,
            model: None,
            mcp_servers: Vec::new(),
            skills: Vec::new(),
            tags: Vec::new(),
            system_prompt: Some(a.system_prompt),
        })
        .collect())
}

async fn build_mcp(
    pool: &PgPool,
    user_id: &systemprompt::identifiers::UserId,
) -> Result<Vec<ManagedMcpServer>, Response> {
    let rows = list_user_mcp_servers(pool, user_id).await.map_err(|e| {
        tracing::error!(error = %e, "list_user_mcp_servers failed");
        shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "MCP listing failed")
    })?;
    Ok(rows
        .into_iter()
        .filter(|m| m.enabled)
        .map(|m| ManagedMcpServer {
            name: m.name,
            url: m.endpoint,
            transport: Some("http".into()),
            oauth: Some(m.oauth_required),
        })
        .collect())
}

fn short_hash(
    plugins: &[PluginEntry],
    mcp: &[ManagedMcpServer],
    skills: &[SkillEntry],
    agents: &[AgentEntry],
) -> String {
    let mut h = Sha256::new();
    if let Ok(s) = serde_json::to_string(plugins) {
        h.update(s.as_bytes());
    }
    h.update(b"|");
    if let Ok(s) = serde_json::to_string(mcp) {
        h.update(s.as_bytes());
    }
    h.update(b"|");
    if let Ok(s) = serde_json::to_string(skills) {
        h.update(s.as_bytes());
    }
    h.update(b"|");
    if let Ok(s) = serde_json::to_string(agents) {
        h.update(s.as_bytes());
    }
    let digest = h.finalize();
    plugin_walker::hex_encode(&digest[..4])
}
