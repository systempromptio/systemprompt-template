use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use systemprompt::database::DbPool;
use systemprompt::identifiers::{McpExecutionId, McpServerId, UserId};
use systemprompt::mcp::McpError;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::artifacts::TextArtifact;
use systemprompt::models::execution::context::RequestContext;

use crate::tools::shared;

#[derive(Deserialize, JsonSchema)]
pub struct UpdateMcpServerInput {
    pub mcp_server_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub endpoint: Option<String>,
    pub binary: Option<String>,
    pub package_name: Option<String>,
    pub port: Option<i32>,
    pub enabled: Option<bool>,
    pub oauth_required: Option<bool>,
    pub oauth_scopes: Option<Vec<String>>,
    pub oauth_audience: Option<String>,
}

pub struct UpdateMcpServerHandler {
    pub db_pool: DbPool,
}

#[async_trait]
impl McpToolHandler for UpdateMcpServerHandler {
    type Input = UpdateMcpServerInput;
    type Output = TextArtifact;

    fn tool_name(&self) -> &'static str {
        "update_mcp_server"
    }

    fn description(&self) -> &'static str {
        "Update an existing MCP server configuration. Requires mcp_server_id. All other \
         fields (name, description, endpoint, enabled, etc.) are optional - only provided \
         fields will be updated."
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

        let update_req = systemprompt_web_extension::admin::types::UpdateUserMcpServerRequest {
            name: input.name,
            description: input.description,
            binary: input.binary,
            package_name: input.package_name,
            port: input.port,
            endpoint: input.endpoint,
            enabled: input.enabled,
            oauth_required: input.oauth_required,
            oauth_scopes: input.oauth_scopes,
            oauth_audience: input.oauth_audience,
        };

        let user_id = UserId::new(ctx.user_id().to_string());
        let mcp_server_id = McpServerId::new(&input.mcp_server_id);
        let server = systemprompt_web_extension::admin::repositories::user_mcp_servers::update_user_mcp_server(
            &pool,
            &user_id,
            &mcp_server_id,
            &update_req,
        )
        .await
        .map_err(|e| McpError::internal_error(format!("Failed to update MCP server: {e}"), None))?
        .ok_or_else(|| {
            McpError::invalid_params(
                format!("MCP server '{}' not found or does not belong to you", input.mcp_server_id), None
            )
        })?;

        shared::invalidate_marketplace_cache(&pool, &user_id).await;

        let result_json = serde_json::to_string_pretty(&serde_json::json!({
            "_display": { "type": "card", "entity": "mcp_server", "action": "updated" },
            "mcp_server_id": server.mcp_server_id,
            "name": server.name,
            "description": server.description,
            "endpoint": server.endpoint,
            "binary": server.binary,
            "package_name": server.package_name,
            "port": server.port,
            "enabled": server.enabled,
            "oauth_required": server.oauth_required,
            "oauth_scopes": server.oauth_scopes,
            "oauth_audience": server.oauth_audience,
            "base_mcp_server_id": server.base_mcp_server_id,
            "created_at": server.created_at.to_rfc3339(),
            "updated_at": server.updated_at.to_rfc3339(),
        }))
        .unwrap_or_default();

        let summary = format!(
            "Updated MCP server '{}' ({})",
            server.name, server.mcp_server_id
        );
        let content = format!("{summary}\n\n{result_json}");
        let artifact =
            TextArtifact::new(&result_json, ctx).with_title(format!("MCP Server: {}", server.name));

        Ok((artifact, content))
    }
}
