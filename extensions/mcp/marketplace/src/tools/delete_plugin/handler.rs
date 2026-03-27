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
pub struct DeletePluginInput {
    pub plugin_id: String,
}

pub struct DeletePluginHandler {
    pub db_pool: DbPool,
}

#[async_trait]
impl McpToolHandler for DeletePluginHandler {
    type Input = DeletePluginInput;
    type Output = TextArtifact;

    fn tool_name(&self) -> &'static str {
        "delete_plugin"
    }

    fn description(&self) -> &'static str {
        "Delete a user plugin and cascade-delete all associated skills, agents, \
         and MCP servers. Requires plugin_id."
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

        let user_id = systemprompt::identifiers::UserId::new(ctx.user_id().to_string());
        let deleted =
            systemprompt_web_extension::admin::repositories::user_plugins::delete_user_plugin(
                &pool,
                &user_id,
                &input.plugin_id,
            )
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to delete plugin: {e}"), None))?;

        if !deleted {
            return Err(McpError::invalid_params(
                format!(
                    "Plugin '{}' not found or does not belong to you",
                    input.plugin_id
                ),
                None,
            ));
        }

        shared::invalidate_marketplace_cache(&pool, &user_id).await;

        let result_json = serde_json::to_string_pretty(&serde_json::json!({
            "_display": { "type": "confirmation", "action": "deleted" },
            "deleted": true,
            "plugin_id": input.plugin_id,
        }))
        .map_err(|e| McpError::internal_error(format!("Failed to serialize result: {e}"), None))?;

        let summary = format!("Deleted plugin '{}'", input.plugin_id);
        let content = format!("{summary}\n\n{result_json}");
        let artifact = TextArtifact::new(&result_json, ctx).with_title("Plugin Deleted");

        Ok((artifact, content))
    }
}
