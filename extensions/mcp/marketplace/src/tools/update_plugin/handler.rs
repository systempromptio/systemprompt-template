use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use systemprompt::database::DbPool;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::McpError;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::artifacts::TextArtifact;
use systemprompt::models::execution::context::RequestContext;

use crate::tools::shared;

#[derive(Deserialize, JsonSchema)]
pub struct UpdatePluginInput {
    pub plugin_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub enabled: Option<bool>,
    pub category: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub author_name: Option<String>,
    pub skill_ids: Option<Vec<String>>,
    pub agent_ids: Option<Vec<String>>,
    pub mcp_server_ids: Option<Vec<String>>,
}

pub struct UpdatePluginHandler {
    pub db_pool: DbPool,
}

#[async_trait]
impl McpToolHandler for UpdatePluginHandler {
    type Input = UpdatePluginInput;
    type Output = TextArtifact;

    fn tool_name(&self) -> &'static str {
        "update_plugin"
    }

    fn description(&self) -> &'static str {
        "Update an existing user plugin. Requires plugin_id. All other fields \
         are optional. Providing skill_ids, agent_ids, or mcp_server_ids replaces \
         existing associations."
    }

    async fn handle(
        &self,
        input: Self::Input,
        ctx: &RequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        let pool = self.db_pool.write_pool().ok_or_else(|| {
            McpError::internal_error("Database pool not available".to_string(), None)
        })?;

        let update_req = systemprompt_web_extension::admin::types::UpdateUserPluginRequest {
            name: input.name,
            description: input.description,
            version: input.version,
            enabled: input.enabled,
            category: input.category,
            keywords: input.keywords,
            author_name: input.author_name,
        };

        let user_id = ctx.user_id().to_string();
        let plugin =
            systemprompt_web_extension::admin::repositories::user_plugins::update_user_plugin(
                &pool,
                &user_id,
                &input.plugin_id,
                &update_req,
            )
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to update plugin: {e}"), None))?
            .ok_or_else(|| {
                McpError::invalid_params(
                    format!(
                        "Plugin '{}' not found or does not belong to you",
                        input.plugin_id
                    ),
                    None,
                )
            })?;

        if let Some(ref skill_slugs) = input.skill_ids {
            let uuids = shared::resolve_skill_slugs(&pool, &user_id, skill_slugs).await?;
            systemprompt_web_extension::admin::repositories::set_plugin_skills(
                &pool, &plugin.id, &uuids,
            )
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to set plugin skills: {e}"), None)
            })?;
        }
        if let Some(ref agent_slugs) = input.agent_ids {
            let uuids = shared::resolve_agent_slugs(&pool, &user_id, agent_slugs).await?;
            systemprompt_web_extension::admin::repositories::set_plugin_agents(
                &pool, &plugin.id, &uuids,
            )
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to set plugin agents: {e}"), None)
            })?;
        }
        if let Some(ref mcp_server_slugs) = input.mcp_server_ids {
            let uuids = shared::resolve_mcp_server_slugs(&pool, &user_id, mcp_server_slugs).await?;
            systemprompt_web_extension::admin::repositories::set_plugin_mcp_servers(
                &pool, &plugin.id, &uuids,
            )
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to set plugin MCP servers: {e}"), None)
            })?;
        }

        let (skill_slugs, agent_slugs, mcp_server_slugs) = if let Ok(Some(assoc)) =
            systemprompt_web_extension::admin::repositories::get_plugin_with_associations(
                &pool,
                &user_id,
                &input.plugin_id,
            )
            .await
        {
            let skills = shared::resolve_skill_uuids_to_slugs(&pool, &assoc.skill_ids).await;
            let agents = shared::resolve_agent_uuids_to_slugs(&pool, &assoc.agent_ids).await;
            let mcps =
                shared::resolve_mcp_server_uuids_to_slugs(&pool, &assoc.mcp_server_ids).await;
            (skills, agents, mcps)
        } else {
            (vec![], vec![], vec![])
        };

        shared::invalidate_marketplace_cache(&pool, &user_id).await;

        let plugin_json = serde_json::to_string_pretty(&serde_json::json!({
            "_display": { "type": "card", "entity": "plugin", "action": "updated" },
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
        .unwrap_or_default();

        let summary = format!("Updated plugin '{}' ({})", plugin.name, plugin.plugin_id);
        let content = format!("{summary}\n\n{plugin_json}");
        let artifact =
            TextArtifact::new(&plugin_json, ctx).with_title(format!("Plugin: {}", plugin.name));

        Ok((artifact, content))
    }
}
