use crate::admin::repositories;
use crate::admin::types::CreateUserPluginRequest;
use sqlx::PgPool;
use systemprompt::identifiers::{AgentId, McpServerId, SkillId, UserId};

#[derive(Debug, Clone)]
pub struct ForkSinglePluginResult {
    pub plugin: crate::admin::types::UserPlugin,
    pub forked_skills: usize,
    pub forked_agents: usize,
}

pub async fn fork_single_plugin(
    pool: &PgPool,
    user_id: &UserId,
    username: &str,
    org_plugin: &crate::admin::types::PluginOverview,
    services_path: &std::path::Path,
    plugin_id_override: Option<String>,
) -> Result<ForkSinglePluginResult, String> {
    let plugin_id = plugin_id_override.unwrap_or_else(|| org_plugin.id.clone());

    let create_plugin_req = CreateUserPluginRequest {
        plugin_id: plugin_id.clone(),
        name: org_plugin.name.clone(),
        description: org_plugin.description.clone(),
        version: "1.0.0".to_string(),
        category: String::new(),
        keywords: vec![],
        author_name: username.to_string(),
        base_plugin_id: Some(org_plugin.id.clone()),
    };

    let plugin = repositories::create_user_plugin(pool, user_id, &create_plugin_req)
        .await
        .map_err(|e| format!("Failed to create forked plugin: {e}"))?;

    let forked_skill_ids = fork_plugin_skills(pool, user_id, org_plugin, services_path).await;
    let forked_agent_ids = fork_plugin_agents(pool, user_id, org_plugin, services_path).await;
    let forked_mcp_ids = fork_plugin_mcp_servers(pool, user_id, org_plugin, services_path).await;

    link_forked_entities(pool, &plugin.id, &forked_skill_ids, &forked_agent_ids, &forked_mcp_ids)
        .await;

    Ok(ForkSinglePluginResult {
        forked_skills: forked_skill_ids.len(),
        forked_agents: forked_agent_ids.len(),
        plugin,
    })
}

async fn link_forked_entities(
    pool: &PgPool,
    plugin_id: &str,
    forked_skill_ids: &[String],
    forked_agent_ids: &[String],
    forked_mcp_ids: &[String],
) {
    let skill_ids: Vec<SkillId> = forked_skill_ids
        .iter()
        .map(|s| SkillId::from(s.clone()))
        .collect();
    let agent_ids: Vec<AgentId> = forked_agent_ids
        .iter()
        .map(|s| AgentId::from(s.clone()))
        .collect();
    if let Err(e) = repositories::set_plugin_skills(pool, plugin_id, &skill_ids).await {
        tracing::warn!(error = %e, "Failed to set plugin skills");
    }
    if let Err(e) = repositories::set_plugin_agents(pool, plugin_id, &agent_ids).await {
        tracing::warn!(error = %e, "Failed to set plugin agents");
    }
    if forked_mcp_ids.is_empty() {
        return;
    }
    let mcp_ids: Vec<McpServerId> = forked_mcp_ids
        .iter()
        .map(|s| McpServerId::new(s.clone()))
        .collect();
    if let Err(e) =
        repositories::user_plugins::set_plugin_mcp_servers(pool, plugin_id, &mcp_ids).await
    {
        tracing::warn!(error = %e, "Failed to set plugin MCP servers");
    }
}

pub(super) fn read_skill_content(skill_dir: &std::path::Path) -> String {
    let skill_md = skill_dir.join("SKILL.md");
    let index_md = skill_dir.join("index.md");
    if skill_md.exists() {
        std::fs::read_to_string(&skill_md).unwrap_or_else(|e| {
            tracing::warn!(error = %e, path = %skill_md.display(), "Failed to read SKILL.md for fork");
            String::new()
        })
    } else if index_md.exists() {
        std::fs::read_to_string(&index_md).unwrap_or_else(|e| {
            tracing::warn!(error = %e, path = %index_md.display(), "Failed to read index.md for fork");
            String::new()
        })
    } else {
        String::new()
    }
}

async fn fork_plugin_skills(
    pool: &PgPool,
    user_id: &UserId,
    org_plugin: &crate::admin::types::PluginOverview,
    services_path: &std::path::Path,
) -> Vec<String> {
    let mut forked_skill_ids = Vec::new();

    for skill_info in &org_plugin.skills {
        let skill_dir = services_path.join("skills").join(skill_info.id.as_str());
        let content = read_skill_content(&skill_dir);
        let create_req = crate::admin::types::CreateSkillRequest {
            skill_id: skill_info.id.clone().into(),
            name: skill_info.name.clone(),
            description: skill_info.description.clone(),
            content,
            tags: vec![],
            base_skill_id: Some(skill_info.id.clone().into()),
        };
        match repositories::get_or_create_user_skill(pool, user_id, &create_req).await {
            Ok(s) => forked_skill_ids.push(s.id),
            Err(e) => tracing::warn!(
                error = %e,
                skill = %skill_info.id,
                "Failed to fork skill during plugin fork"
            ),
        }
    }

    forked_skill_ids
}

async fn fork_plugin_agents(
    pool: &PgPool,
    user_id: &UserId,
    org_plugin: &crate::admin::types::PluginOverview,
    services_path: &std::path::Path,
) -> Vec<String> {
    let agents_path = services_path.join("agents");
    let mut forked_agent_ids = Vec::new();

    for agent_info in &org_plugin.agents {
        let (name, description, system_prompt) =
            super::read_agent_from_fs(&agents_path, agent_info.id.as_str());
        let create_req = crate::admin::types::CreateUserAgentRequest {
            agent_id: agent_info.id.clone().into(),
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
            base_agent_id: Some(agent_info.id.clone().into()),
        };
        match repositories::get_or_create_user_agent(pool, user_id, &create_req).await {
            Ok(a) => forked_agent_ids.push(a.id),
            Err(e) => tracing::warn!(
                error = %e,
                agent = %agent_info.id,
                "Failed to fork agent during plugin fork"
            ),
        }
    }

    forked_agent_ids
}

async fn fork_plugin_mcp_servers(
    pool: &PgPool,
    user_id: &UserId,
    org_plugin: &crate::admin::types::PluginOverview,
    services_path: &std::path::Path,
) -> Vec<String> {
    let mut forked_mcp_ids = Vec::new();

    for mcp_server_id in &org_plugin.mcp_servers {
        if let Some(id) = fork_single_mcp_server(pool, user_id, services_path, mcp_server_id).await
        {
            forked_mcp_ids.push(id);
        }
    }

    forked_mcp_ids
}

async fn fork_single_mcp_server(
    pool: &PgPool,
    user_id: &UserId,
    services_path: &std::path::Path,
    mcp_server_id: &str,
) -> Option<String> {
    let server_detail = match repositories::mcp_servers::find_mcp_server(services_path, mcp_server_id) {
        Ok(Some(s)) => s,
        Ok(None) => {
            tracing::warn!(mcp_server = %mcp_server_id, "MCP server not found during plugin fork, skipping");
            return None;
        }
        Err(e) => {
            tracing::warn!(error = %e, mcp_server = %mcp_server_id, "Failed to read MCP server config during plugin fork");
            return None;
        }
    };

    let create_req = crate::admin::types::CreateUserMcpServerRequest {
        mcp_server_id: McpServerId::new(mcp_server_id.to_string()),
        name: server_detail.description.clone(),
        description: server_detail.description.clone(),
        binary: server_detail.binary,
        package_name: server_detail.package_name,
        port: i32::from(server_detail.port),
        endpoint: server_detail.endpoint,
        oauth_required: server_detail.oauth_required,
        oauth_scopes: server_detail.oauth_scopes,
        oauth_audience: server_detail.oauth_audience,
        base_mcp_server_id: Some(McpServerId::new(mcp_server_id.to_string())),
    };

    match repositories::user_mcp_servers::get_or_create_user_mcp_server(pool, user_id, &create_req)
        .await
    {
        Ok(mcp) => Some(mcp.id),
        Err(e) => {
            tracing::warn!(error = %e, mcp_server = %mcp_server_id, "Failed to fork MCP server during plugin fork");
            None
        }
    }
}
