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
pub struct GetPluginInput {
    pub plugin_id: String,
}

pub struct GetPluginHandler {
    pub db_pool: DbPool,
}

#[async_trait]
impl McpToolHandler for GetPluginHandler {
    type Input = GetPluginInput;
    type Output = TextArtifact;

    fn tool_name(&self) -> &'static str {
        "get_plugin"
    }

    fn description(&self) -> &'static str {
        "Get a single plugin with its associations (skill IDs, agent IDs, \
         MCP server IDs). Requires plugin_id."
    }

    async fn handle(
        &self,
        input: Self::Input,
        ctx: &RequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        let pool = self.db_pool.pool().ok_or_else(|| {
            McpError::internal_error("Database pool not available".to_string(), None)
        })?;

        let user_id = UserId::new(ctx.user_id().to_string());
        let assoc = systemprompt_web_extension::admin::repositories::get_plugin_with_associations(
            &pool,
            &user_id,
            &input.plugin_id,
        )
        .await
        .map_err(|e| McpError::internal_error(format!("Failed to get plugin: {e}"), None))?
        .ok_or_else(|| {
            McpError::invalid_params(
                format!(
                    "Plugin '{}' not found or does not belong to you",
                    input.plugin_id
                ),
                None,
            )
        })?;

        let plugin = &assoc.plugin;

        let skill_strs: Vec<String> = assoc.skill_ids.iter().map(std::string::ToString::to_string).collect();
        let agent_strs: Vec<String> = assoc.agent_ids.iter().map(std::string::ToString::to_string).collect();
        let mcp_strs: Vec<String> = assoc.mcp_server_ids.iter().map(std::string::ToString::to_string).collect();
        let skill_slugs = shared::resolve_skill_uuids_to_slugs(&pool, &skill_strs).await;
        let agent_slugs = shared::resolve_agent_uuids_to_slugs(&pool, &agent_strs).await;
        let mcp_server_slugs =
            shared::resolve_mcp_server_uuids_to_slugs(&pool, &mcp_strs).await;

        let plugin_json = serde_json::to_string_pretty(&serde_json::json!({
            "_display": { "type": "card", "entity": "plugin", "action": "retrieved" },
            "plugin_id": plugin.plugin_id,
            "name": plugin.name,
            "description": plugin.description,
            "version": plugin.version,
            "enabled": plugin.enabled,
            "category": plugin.category,
            "keywords": plugin.keywords,
            "author_name": plugin.author_name,
            "base_plugin_id": plugin.base_plugin_id,
            "skill_ids": skill_slugs,
            "agent_ids": agent_slugs,
            "mcp_server_ids": mcp_server_slugs,
            "created_at": plugin.created_at.to_rfc3339(),
            "updated_at": plugin.updated_at.to_rfc3339(),
        }))
        .map_err(|e| McpError::internal_error(format!("Failed to serialize result: {e}"), None))?;

        let summary = format!("Retrieved plugin '{}' ({})", plugin.name, plugin.plugin_id);
        let content = format!("{summary}\n\n{plugin_json}");
        let artifact =
            TextArtifact::new(&plugin_json, ctx).with_title(format!("Plugin: {}", plugin.name));

        Ok((artifact, content))
    }
}
