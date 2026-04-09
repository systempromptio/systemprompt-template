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

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteAgentInput {
    pub agent_id: String,
}

#[derive(Debug)]
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
        let pool = shared::require_write_pool(&self.db_pool)?;

        let user_id = systemprompt::identifiers::UserId::new(ctx.user_id().to_string());
        let agent_id = systemprompt::identifiers::AgentId::new(&input.agent_id);
        let deleted =
            systemprompt_web_extension::admin::repositories::user_agents::delete_user_agent(
                &pool, &user_id, &agent_id,
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
        .map_err(|e| McpError::internal_error(format!("Failed to serialize result: {e}"), None))?;

        let summary = format!("Deleted agent '{}'", input.agent_id);
        let content = format!("{summary}\n\n{result_json}");
        let artifact = TextArtifact::new(&result_json, ctx).with_title("Agent Deleted");

        Ok((artifact, content))
    }
}
