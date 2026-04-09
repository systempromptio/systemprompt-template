use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use systemprompt::database::DbPool;
use systemprompt::identifiers::{McpExecutionId, UserId};
use systemprompt::mcp::McpError;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::artifacts::{Column, ColumnType, TableArtifact};
use systemprompt::models::execution::context::RequestContext;

use crate::tools::shared;

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct ListMcpServersInput;

#[derive(Debug)]
pub struct ListMcpServersHandler {
    pub db_pool: DbPool,
}

#[async_trait]
impl McpToolHandler for ListMcpServersHandler {
    type Input = ListMcpServersInput;
    type Output = TableArtifact;

    fn tool_name(&self) -> &'static str {
        "list_mcp_servers"
    }

    fn description(&self) -> &'static str {
        "List all MCP servers for the authenticated user. Returns server metadata \
         including endpoints and connection details."
    }

    async fn handle(
        &self,
        _input: Self::Input,
        ctx: &RequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        let pool = shared::require_pool(&self.db_pool)?;
        let user_id = UserId::new(ctx.user_id().to_string());

        let servers =
            systemprompt_web_extension::admin::repositories::user_mcp_servers::list_user_mcp_servers(
                &pool, &user_id,
            )
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to list MCP servers: {e}"), None)
            })?;

        let columns = vec![
            Column::new("mcp_server_id", ColumnType::String),
            Column::new("name", ColumnType::String),
            Column::new("description", ColumnType::String),
            Column::new("endpoint", ColumnType::String),
            Column::new("binary", ColumnType::String),
            Column::new("package_name", ColumnType::String),
            Column::new("port", ColumnType::Integer),
            Column::new("enabled", ColumnType::Boolean),
            Column::new("oauth_required", ColumnType::Boolean),
            Column::new("base_mcp_server_id", ColumnType::String),
            Column::new("created_at", ColumnType::Date),
            Column::new("updated_at", ColumnType::Date),
        ];

        let rows: Vec<serde_json::Value> = servers
            .iter()
            .map(|s| {
                serde_json::json!({
                    "mcp_server_id": s.mcp_server_id,
                    "name": s.name,
                    "description": s.description,
                    "endpoint": s.endpoint,
                    "binary": s.binary,
                    "package_name": s.package_name,
                    "port": s.port,
                    "enabled": s.enabled,
                    "oauth_required": s.oauth_required,
                    "base_mcp_server_id": s.base_mcp_server_id,
                    "created_at": s.created_at.to_rfc3339(),
                    "updated_at": s.updated_at.to_rfc3339(),
                })
            })
            .collect();

        let total = rows.len();
        let summary = format!("Found {total} MCP server(s) for user '{user_id}'");
        let artifact = TableArtifact::new(columns, ctx).with_rows(rows);

        Ok((artifact, summary))
    }
}
