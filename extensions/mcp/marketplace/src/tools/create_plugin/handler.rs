use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use systemprompt::database::DbPool;
use systemprompt::identifiers::{McpExecutionId, UserId};
use systemprompt::mcp::McpError;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::artifacts::TextArtifact;
use systemprompt::models::execution::context::RequestContext;

use crate::tools::shared;

#[derive(Deserialize, JsonSchema)]
pub struct CreatePluginInput {
    pub name: String,
    pub description: String,
    pub category: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    pub version: Option<String>,
    pub author_name: Option<String>,
    #[serde(default)]
    pub skill_ids: Vec<String>,
    #[serde(default)]
    pub agent_ids: Vec<String>,
    #[serde(default)]
    pub mcp_server_ids: Vec<String>,
}

pub struct CreatePluginHandler {
    pub db_pool: DbPool,
}

#[async_trait]
impl McpToolHandler for CreatePluginHandler {
    type Input = CreatePluginInput;
    type Output = TextArtifact;

    fn tool_name(&self) -> &'static str {
        "create_plugin"
    }

    fn description(&self) -> &'static str {
        "Create a new user plugin. Requires name and description. \
         Optionally provide category, keywords, version, author_name, and \
         association arrays (skill_ids, agent_ids, mcp_server_ids)."
    }

    async fn handle(
        &self,
        input: Self::Input,
        ctx: &RequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        let plugin_id = shared::generate_slug(&input.name);

        let pool = self.db_pool.write_pool().ok_or_else(|| {
            McpError::internal_error("Database pool not available".to_string(), None)
        })?;

        let create_req = systemprompt_web_extension::admin::types::CreateUserPluginRequest {
            plugin_id: plugin_id.clone(),
            name: input.name.clone(),
            description: input.description.clone(),
            version: input.version.unwrap_or_else(|| "1.0.0".to_string()),
            category: input.category.unwrap_or_else(String::new),
            keywords: input.keywords.clone(),
            author_name: input.author_name.unwrap_or_else(String::new),
            base_plugin_id: None,
        };

        let user_id = UserId::new(ctx.user_id().to_string());
        let plugin =
            systemprompt_web_extension::admin::repositories::user_plugins::create_user_plugin(
                &pool,
                &user_id,
                &create_req,
            )
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to create plugin: {e}"), None))?;

        if !input.skill_ids.is_empty() {
            let uuids =
                shared::resolve_skill_slugs(&pool, user_id.as_ref(), &input.skill_ids).await?;
            let typed: Vec<systemprompt::identifiers::SkillId> = uuids
                .into_iter()
                .map(systemprompt::identifiers::SkillId::new)
                .collect();
            systemprompt_web_extension::admin::repositories::set_plugin_skills(
                &pool, &plugin.id, &typed,
            )
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to set plugin skills: {e}"), None)
            })?;
        }
        if !input.agent_ids.is_empty() {
            let uuids =
                shared::resolve_agent_slugs(&pool, user_id.as_ref(), &input.agent_ids).await?;
            let typed: Vec<systemprompt::identifiers::AgentId> = uuids
                .into_iter()
                .map(systemprompt::identifiers::AgentId::new)
                .collect();
            systemprompt_web_extension::admin::repositories::set_plugin_agents(
                &pool, &plugin.id, &typed,
            )
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to set plugin agents: {e}"), None)
            })?;
        }
        if !input.mcp_server_ids.is_empty() {
            let uuids =
                shared::resolve_mcp_server_slugs(&pool, user_id.as_ref(), &input.mcp_server_ids)
                    .await?;
            let typed: Vec<systemprompt::identifiers::McpServerId> = uuids
                .into_iter()
                .map(systemprompt::identifiers::McpServerId::new)
                .collect();
            systemprompt_web_extension::admin::repositories::set_plugin_mcp_servers(
                &pool, &plugin.id, &typed,
            )
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to set plugin MCP servers: {e}"), None)
            })?;
        }

        let (skill_slugs, agent_slugs, mcp_server_slugs) =
            shared::resolve_association_slugs(&pool, &user_id, &plugin.plugin_id).await;

        shared::invalidate_marketplace_cache(&pool, &user_id).await;

        let plugin_json = serde_json::to_string_pretty(&serde_json::json!({
            "_display": { "type": "card", "entity": "plugin", "action": "created" },
            "plugin_id": plugin.plugin_id,
            "name": plugin.name,
            "description": plugin.description,
            "version": plugin.version,
            "enabled": plugin.enabled,
            "category": plugin.category,
            "keywords": plugin.keywords,
            "author_name": plugin.author_name,
            "skill_ids": skill_slugs,
            "agent_ids": agent_slugs,
            "mcp_server_ids": mcp_server_slugs,
            "created_at": plugin.created_at.to_rfc3339(),
            "updated_at": plugin.updated_at.to_rfc3339(),
        }))
        .map_err(|e| McpError::internal_error(format!("Failed to serialize plugin: {e}"), None))?;

        let summary = format!("Created plugin '{}' ({})", plugin.name, plugin.plugin_id);
        let content = format!("{summary}\n\n{plugin_json}");
        let artifact =
            TextArtifact::new(&plugin_json, ctx).with_title(format!("Plugin: {}", plugin.name));

        Ok((artifact, content))
    }
}
