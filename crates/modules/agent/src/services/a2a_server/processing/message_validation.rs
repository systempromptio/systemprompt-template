use anyhow::{anyhow, Result};
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use systemprompt_core_system::RequestContext;
use systemprompt_identifiers::{ContextId, TaskId};

use crate::models::{AgentRuntimeInfo, Message};
use crate::repository::{ArtifactRepository, ContextRepository, TaskRepository};
use crate::services::registry::AgentRegistry;

#[derive(Clone, Debug)]
pub struct ValidatedMessageRequest {
    pub message: Message,
    pub agent_name: String,
    pub context_id: ContextId,
    pub task_id: TaskId,
    pub agent_runtime: AgentRuntimeInfo,
    pub has_tools: bool,
}

#[derive(Debug)]
pub struct MessageValidationService {
    db_pool: DbPool,
}

impl MessageValidationService {
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    pub async fn validate_message_request(
        &self,
        message: &Message,
        agent_name: &str,
        context: &RequestContext,
    ) -> Result<ValidatedMessageRequest> {
        self.validate_message_format(message)?;

        let agent_runtime = self.load_agent_runtime(agent_name).await?;

        self.validate_context_ownership(message, context).await?;

        let task_id = self.determine_task_id(message);

        let has_tools = !agent_runtime.mcp_servers.is_empty();

        Ok(ValidatedMessageRequest {
            message: message.clone(),
            agent_name: agent_name.to_string(),
            context_id: message.context_id.clone(),
            task_id,
            agent_runtime,
            has_tools,
        })
    }

    async fn load_agent_runtime(&self, agent_name: &str) -> Result<AgentRuntimeInfo> {
        let registry = AgentRegistry::new().await?;
        let agent_config = registry
            .get_agent(agent_name)
            .await
            .map_err(|_| anyhow!("Agent not found: {}", agent_name))?;

        Ok(agent_config.into())
    }

    async fn validate_context_ownership(
        &self,
        message: &Message,
        context: &RequestContext,
    ) -> Result<()> {
        let task_repo = Arc::new(TaskRepository::new(self.db_pool.clone()));
        let artifact_repo = Arc::new(ArtifactRepository::new(self.db_pool.clone()));
        let context_repo = ContextRepository::new(self.db_pool.clone(), task_repo, artifact_repo);

        context_repo
            .get_context(message.context_id.as_str(), context.user_id().as_str())
            .await
            .map_err(|e| {
                anyhow!(
                    "Context validation failed - context_id: {}, user_id: {}, error: {}",
                    message.context_id,
                    context.user_id(),
                    e
                )
            })?;

        Ok(())
    }

    fn validate_message_format(&self, message: &Message) -> Result<()> {
        let has_text_part = message
            .parts
            .iter()
            .any(|part| matches!(part, crate::models::Part::Text(_)));

        if !has_text_part {
            return Err(anyhow!("No text content found in message"));
        }

        Ok(())
    }

    fn determine_task_id(&self, message: &Message) -> TaskId {
        if let Some(existing_task_id) = message.task_id.clone() {
            existing_task_id
        } else {
            TaskId::new(uuid::Uuid::new_v4().to_string())
        }
    }
}
