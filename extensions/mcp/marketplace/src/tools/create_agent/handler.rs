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
pub struct CreateAgentInput {
    pub name: String,
    pub description: String,
    pub system_prompt: String,
    pub base_agent_id: Option<String>,
}

pub struct CreateAgentHandler {
    pub db_pool: DbPool,
}

#[async_trait]
impl McpToolHandler for CreateAgentHandler {
    type Input = CreateAgentInput;
    type Output = TextArtifact;

    fn tool_name(&self) -> &'static str {
        "create_agent"
    }

    fn description(&self) -> &'static str {
        "Create a new user agent. Requires name, description, and system_prompt. \
         Optionally provide base_agent_id to fork from an existing agent."
    }

    async fn handle(
        &self,
        input: Self::Input,
        ctx: &RequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        let agent_id = shared::generate_slug(&input.name);

        let pool = self.db_pool.write_pool().ok_or_else(|| {
            McpError::internal_error("Database pool not available".to_string(), None)
        })?;

        let create_req = systemprompt_web_extension::admin::types::CreateUserAgentRequest {
            agent_id: systemprompt::identifiers::AgentId::new(agent_id.clone()),
            name: input.name.clone(),
            description: input.description.clone(),
            system_prompt: input.system_prompt.clone(),
            base_agent_id: input.base_agent_id.map(systemprompt::identifiers::AgentId::new),
        };

        let user_id = UserId::new(ctx.user_id().to_string());
        let agent =
            systemprompt_web_extension::admin::repositories::user_agents::create_user_agent(
                &pool,
                &user_id,
                &create_req,
            )
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to create agent: {e}"), None))?;

        let added_to_plugin =
            shared::auto_add_to_default_plugin(&self.db_pool, &user_id, &agent.id, "agent").await;

        shared::invalidate_marketplace_cache(&pool, &user_id).await;

        let agent_json = serde_json::to_string_pretty(&serde_json::json!({
            "_display": { "type": "card", "entity": "agent", "action": "created" },
            "agent_id": agent.agent_id,
            "name": agent.name,
            "description": agent.description,
            "system_prompt": agent.system_prompt,
            "enabled": agent.enabled,
            "base_agent_id": agent.base_agent_id,
            "added_to_plugin": added_to_plugin,
            "created_at": agent.created_at.to_rfc3339(),
            "updated_at": agent.updated_at.to_rfc3339(),
        }))
        .map_err(|e| McpError::internal_error(format!("Failed to serialize agent: {e}"), None))?;

        let summary = if let Some(ref plugin_id) = added_to_plugin {
            format!(
                "Created agent '{}' ({}) and added to plugin '{}'",
                agent.name, agent.agent_id, plugin_id
            )
        } else {
            format!("Created agent '{}' ({})", agent.name, agent.agent_id)
        };
        let content = format!("{summary}\n\n{agent_json}");
        let artifact =
            TextArtifact::new(&agent_json, ctx).with_title(format!("Agent: {}", agent.name));

        Ok((artifact, content))
    }
}
