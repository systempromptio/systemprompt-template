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
pub struct DeleteAgentInput {
    pub agent_id: String,
}

pub struct DeleteAgentHandler {
    pub db_pool: DbPool,
}

#[async_trait]
impl McpToolHandler for DeleteAgentHandler {
    type Input = DeleteAgentInput;
    type Output = TextArtifact;

    fn tool_name(&self) -> &'static str {
        "delete_agent"
    }

    fn description(&self) -> &'static str {
        "Delete a user agent. Requires agent_id. Returns whether the agent \
         was successfully deleted."
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

        let user_id = ctx.user_id().to_string();
        let deleted =
            systemprompt_web_extension::admin::repositories::user_agents::delete_user_agent(
                &pool,
                &user_id,
                &input.agent_id,
            )
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to delete agent: {e}"), None))?;

        if !deleted {
            return Err(McpError::invalid_params(
                format!(
                    "Agent '{}' not found or does not belong to you",
                    input.agent_id
                ),
                None,
            ));
        }

        shared::invalidate_marketplace_cache(&pool, &user_id).await;

        let result_json = serde_json::to_string_pretty(&serde_json::json!({
            "_display": { "type": "confirmation", "action": "deleted" },
            "deleted": true,
            "agent_id": input.agent_id,
        }))
        .unwrap_or_default();

        let summary = format!("Deleted agent '{}'", input.agent_id);
        let content = format!("{summary}\n\n{result_json}");
        let artifact = TextArtifact::new(&result_json, ctx).with_title("Agent Deleted");

        Ok((artifact, content))
    }
}
