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
pub struct CreateMcpServerInput {
    pub name: String,
    pub description: String,
    pub endpoint: String,
    pub plugin_id: Option<String>,
    pub binary: Option<String>,
    pub package_name: Option<String>,
    #[serde(default)]
    pub port: i32,
    #[serde(default)]
    pub oauth_required: bool,
    #[serde(default)]
    pub oauth_scopes: Vec<String>,
    pub oauth_audience: Option<String>,
    pub base_mcp_server_id: Option<String>,
}

pub struct CreateMcpServerHandler {
    pub db_pool: DbPool,
}

#[async_trait]
impl McpToolHandler for CreateMcpServerHandler {
    type Input = CreateMcpServerInput;
    type Output = TextArtifact;

    fn tool_name(&self) -> &'static str {
        "create_mcp_server"
    }

    fn description(&self) -> &'static str {
        "Create a new MCP server configuration. Requires name, description, and endpoint. \
         Optionally provide plugin_id to add to a specific plugin, or it will be added to \
         the default plugin."
    }

    async fn handle(
        &self,
        input: Self::Input,
        ctx: &RequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        let mcp_server_id = shared::generate_slug(&input.name);

        let pool = self.db_pool.write_pool().ok_or_else(|| {
            McpError::internal_error("Database pool not available".to_string(), None)
        })?;

        let create_req = systemprompt_web_extension::admin::types::CreateUserMcpServerRequest {
            mcp_server_id: systemprompt::identifiers::McpServerId::new(mcp_server_id.clone()),
            name: input.name.clone(),
            description: input.description.clone(),
            binary: input.binary.unwrap_or_default(),
            package_name: input.package_name.unwrap_or_default(),
            port: input.port,
            endpoint: input.endpoint.clone(),
            oauth_required: input.oauth_required,
            oauth_scopes: input.oauth_scopes,
            oauth_audience: input.oauth_audience.unwrap_or_default(),
            base_mcp_server_id: input.base_mcp_server_id.map(systemprompt::identifiers::McpServerId::new),
        };

        let user_id = UserId::new(ctx.user_id().to_string());
        let server = systemprompt_web_extension::admin::repositories::user_mcp_servers::create_user_mcp_server(
            &pool,
            &user_id,
            &create_req,
        )
        .await
        .map_err(|e| McpError::internal_error(format!("Failed to create MCP server: {e}"), None))?;

        let added_to_plugin = shared::add_to_plugin(
            &self.db_pool,
            &user_id,
            &server.id,
            "mcp_server",
            input.plugin_id.as_deref(),
        )
        .await;

        shared::invalidate_marketplace_cache(&pool, &user_id).await;

        let result_json = serde_json::to_string_pretty(&serde_json::json!({
            "_display": { "type": "card", "entity": "mcp_server", "action": "created" },
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
            "added_to_plugin": added_to_plugin,
            "created_at": server.created_at.to_rfc3339(),
            "updated_at": server.updated_at.to_rfc3339(),
        }))
        .map_err(|e| McpError::internal_error(format!("Failed to serialize MCP server: {e}"), None))?;

        let summary = if let Some(ref plugin_id) = added_to_plugin {
            format!(
                "Created MCP server '{}' ({}) and added to plugin '{}'",
                server.name, server.mcp_server_id, plugin_id
            )
        } else {
            format!(
                "Created MCP server '{}' ({})",
                server.name, server.mcp_server_id
            )
        };
        let content = format!("{summary}\n\n{result_json}");
        let artifact =
            TextArtifact::new(&result_json, ctx).with_title(format!("MCP Server: {}", server.name));

        Ok((artifact, content))
    }
}
