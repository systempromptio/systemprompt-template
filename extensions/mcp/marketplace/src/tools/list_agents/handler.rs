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

#[derive(Deserialize, JsonSchema)]
pub struct ListAgentsInput {}

pub struct ListAgentsHandler {
    pub db_pool: DbPool,
}

#[async_trait]
impl McpToolHandler for ListAgentsHandler {
    type Input = ListAgentsInput;
    type Output = TableArtifact;

    fn tool_name(&self) -> &'static str {
        "list_agents"
    }

    fn description(&self) -> &'static str {
        "List all agents for the authenticated user. Returns agent metadata \
         including creation dates."
    }

    async fn handle(
        &self,
        _input: Self::Input,
        ctx: &RequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        let pool = shared::require_pool(&self.db_pool)?;
        let user_id = UserId::new(ctx.user_id().to_string());

        let agents =
            systemprompt_web_extension::admin::repositories::user_agents::list_user_agents(
                &pool, &user_id,
            )
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to list agents: {e}"), None))?;

        let columns = vec![
            Column::new("agent_id", ColumnType::String),
            Column::new("name", ColumnType::String),
            Column::new("description", ColumnType::String),
            Column::new("base_agent_id", ColumnType::String),
            Column::new("created_at", ColumnType::Date),
            Column::new("updated_at", ColumnType::Date),
        ];

        let rows: Vec<serde_json::Value> = agents
            .iter()
            .map(|a| {
                serde_json::json!({
                    "agent_id": a.agent_id,
                    "name": a.name,
                    "description": a.description,
                    "base_agent_id": a.base_agent_id,
                    "created_at": a.created_at.to_rfc3339(),
                    "updated_at": a.updated_at.to_rfc3339(),
                })
            })
            .collect();

        let total = rows.len();
        let summary = format!("Found {total} agent(s) for user '{user_id}'");
        let artifact = TableArtifact::new(columns, ctx).with_rows(rows);

        Ok((artifact, summary))
    }
}
