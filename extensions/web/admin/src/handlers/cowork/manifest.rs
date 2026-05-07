use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use systemprompt::security::manifest_signing;

use crate::handlers::shared;

use super::load_user_section;
use super::plugin_walker::{self, PluginEntry};
use super::types::{AgentEntry, ManagedMcpServer, Manifest, SkillEntry};

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
            return shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "User lookup failed");
        }
    };

    let plugins: Vec<PluginEntry> = Vec::new();
    let skills: Vec<SkillEntry> = Vec::new();
    let agents: Vec<AgentEntry> = Vec::new();
    let managed_mcp_servers: Vec<ManagedMcpServer> = Vec::new();

    let mut manifest =
        assemble_manifest(&user_id, user, plugins, skills, agents, managed_mcp_servers);

    match manifest_signing::sign_value(&manifest) {
        Ok(sig) => manifest.signature = Some(sig),
        Err(e) => {
            tracing::error!(error = %e, "manifest signing failed");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Manifest signing failed",
            );
        }
    }

    (StatusCode::OK, Json(manifest)).into_response()
}

fn assemble_manifest(
    user_id: &systemprompt::identifiers::UserId,
    user: super::types::UserSection,
    plugins: Vec<PluginEntry>,
    skills: Vec<SkillEntry>,
    agents: Vec<AgentEntry>,
    managed_mcp_servers: Vec<ManagedMcpServer>,
) -> Manifest {
    let version = format!(
        "{}-{}",
        chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ"),
        short_hash(&plugins, &managed_mcp_servers, &skills, &agents),
    );
    Manifest {
        version,
        issued_at: chrono::Utc::now().to_rfc3339(),
        not_before: chrono::Utc::now().to_rfc3339(),
        user_id: user_id.as_str().to_string(),
        user,
        plugins,
        skills,
        agents,
        managed_mcp_servers,
        revocations: Vec::new(),
        signature: None,
    }
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
