use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use systemprompt::database::DbPool;
use systemprompt::identifiers::{AgentId, McpExecutionId, UserId};
use systemprompt::mcp::McpError;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::artifacts::TextArtifact;
use systemprompt::models::execution::context::RequestContext;

use crate::tools::shared;

const MAX_NAME_LEN: usize = 256;
const MAX_DESCRIPTION_LEN: usize = 4096;
const MAX_SYSTEM_PROMPT_LEN: usize = 65536;

#[derive(Deserialize, JsonSchema)]
pub struct UpdateAgentInput {
    pub agent_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub system_prompt: Option<String>,
}

pub struct UpdateAgentHandler {
    pub db_pool: DbPool,
}

#[async_trait]
impl McpToolHandler for UpdateAgentHandler {
    type Input = UpdateAgentInput;
    type Output = TextArtifact;

    fn tool_name(&self) -> &'static str {
        "update_agent"
    }

    fn description(&self) -> &'static str {
        "Update an existing user agent. Requires agent_id. All other fields \
         (name, description, system_prompt) are optional - only provided \
         fields will be updated."
    }

    async fn handle(
        &self,
        input: Self::Input,
        ctx: &RequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        if let Some(ref name) = input.name {
            if name.len() > MAX_NAME_LEN {
                return Err(McpError::invalid_params(
                    format!("name exceeds maximum length of {MAX_NAME_LEN}"),
                    None,
                ));
            }
        }
        if let Some(ref description) = input.description {
            if description.len() > MAX_DESCRIPTION_LEN {
                return Err(McpError::invalid_params(
                    format!("description exceeds maximum length of {MAX_DESCRIPTION_LEN}"),
                    None,
                ));
            }
        }
        if let Some(ref system_prompt) = input.system_prompt {
            if system_prompt.len() > MAX_SYSTEM_PROMPT_LEN {
                return Err(McpError::invalid_params(
                    format!("system_prompt exceeds maximum length of {MAX_SYSTEM_PROMPT_LEN}"),
                    None,
                ));
            }
        }

        let pool = shared::require_write_pool(&self.db_pool)?;

        let update_req = systemprompt_web_extension::admin::types::UpdateUserAgentRequest {
            name: input.name,
            description: input.description,
            system_prompt: input.system_prompt,
        };

        let user_id = UserId::new(ctx.user_id().to_string());
        let agent_id = AgentId::new(&input.agent_id);
        let agent =
            systemprompt_web_extension::admin::repositories::user_agents::update_user_agent(
                &pool,
                &user_id,
                &agent_id,
                &update_req,
            )
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to update agent: {e}"), None))?
            .ok_or_else(|| {
                McpError::invalid_params(
                    format!(
                        "Agent '{}' not found or does not belong to you",
                        input.agent_id
                    ),
                    None,
                )
            })?;

        shared::invalidate_marketplace_cache(&pool, &user_id).await;

        let agent_json = serde_json::to_string_pretty(&serde_json::json!({
            "_display": { "type": "card", "entity": "agent", "action": "updated" },
            "agent_id": agent.agent_id,
            "name": agent.name,
            "description": agent.description,
            "system_prompt": agent.system_prompt,
            "base_agent_id": agent.base_agent_id,
            "created_at": agent.created_at.to_rfc3339(),
            "updated_at": agent.updated_at.to_rfc3339(),
        }))
        .map_err(|e| McpError::internal_error(format!("Serialization failed: {e}"), None))?;

        let summary = format!("Updated agent '{}' ({})", agent.name, agent.agent_id);
        let content = format!("{summary}\n\n{agent_json}");
        let artifact =
            TextArtifact::new(&agent_json, ctx).with_title(format!("Agent: {}", agent.name));

        Ok((artifact, content))
    }
}
