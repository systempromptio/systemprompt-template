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
pub struct DeleteMcpServerInput {
    pub mcp_server_id: String,
}

pub struct DeleteMcpServerHandler {
    pub db_pool: DbPool,
}

#[async_trait]
impl McpToolHandler for DeleteMcpServerHandler {
    type Input = DeleteMcpServerInput;
    type Output = TextArtifact;

    fn tool_name(&self) -> &'static str {
        "delete_mcp_server"
    }

    fn description(&self) -> &'static str {
        "Delete a user MCP server configuration. Requires mcp_server_id. Returns whether \
         the server was successfully deleted."
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
        let mcp_server_id = systemprompt::identifiers::McpServerId::new(&input.mcp_server_id);
        let deleted =
            systemprompt_web_extension::admin::repositories::user_mcp_servers::delete_user_mcp_server(
                &pool, &user_id, &mcp_server_id,
            )
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to delete MCP server: {e}"), None))?;

        if !deleted {
            return Err(McpError::invalid_params(
                format!(
                    "MCP server '{}' not found or does not belong to you",
                    input.mcp_server_id
                ),
                None,
            ));
        }

        shared::invalidate_marketplace_cache(&pool, &user_id).await;

        let result_json = serde_json::to_string_pretty(&serde_json::json!({
            "_display": { "type": "confirmation", "action": "deleted" },
            "deleted": true,
            "mcp_server_id": input.mcp_server_id,
        }))
        .map_err(|e| McpError::internal_error(format!("Failed to serialize result: {e}"), None))?;

        let summary = format!("Deleted MCP server '{}'", input.mcp_server_id);
        let content = format!("{summary}\n\n{result_json}");
        let artifact = TextArtifact::new(&result_json, ctx).with_title("MCP Server Deleted");

        Ok((artifact, content))
    }
}
