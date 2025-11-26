use anyhow::{anyhow, Result};
use serde_json::json;
use uuid::Uuid;

use crate::models::a2a::{Artifact, Message, Part, TextPart};
use crate::repository::{ArtifactRepository, SkillRepository};
use crate::services::MessageService;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::{LogLevel, LogService};
use systemprompt_core_system::RequestContext;
use systemprompt_identifiers::{ContextId, TaskId, UserId};
use systemprompt_models::execution::CallSource;

pub struct ArtifactPublishingService {
    artifact_repo: ArtifactRepository,
    skill_repo: SkillRepository,
    message_service: MessageService,
    logger: LogService,
}

impl std::fmt::Debug for ArtifactPublishingService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArtifactPublishingService")
            .finish_non_exhaustive()
    }
}

impl ArtifactPublishingService {
    pub fn new(db_pool: DbPool, logger: LogService) -> Self {
        Self {
            artifact_repo: ArtifactRepository::new(db_pool.clone()),
            skill_repo: SkillRepository::new(db_pool.clone()),
            message_service: MessageService::new(db_pool.clone(), logger.clone()),
            logger,
        }
    }

    async fn enrich_artifact_with_skill(&self, artifact: &Artifact) -> Artifact {
        let mut enriched = artifact.clone();

        if let Some(skill_id) = &enriched.metadata.skill_id {
            if enriched.metadata.skill_name.is_none() {
                let skill_id_typed = systemprompt_identifiers::SkillId::new(skill_id);
                if let Ok(Some(skill)) = self.skill_repo.get_by_skill_id(&skill_id_typed).await {
                    enriched.metadata.skill_name = Some(skill.name);
                }
            }
        }

        enriched
    }

    pub async fn publish_from_a2a(
        &self,
        artifact: &Artifact,
        task_id: &TaskId,
        context_id: &ContextId,
        _user_id: &UserId,
    ) -> Result<()> {
        let enriched_artifact = self.enrich_artifact_with_skill(artifact).await;

        self.logger
            .log(
                LogLevel::Info,
                "artifact_publishing",
                "Publishing artifact from A2A agent",
                Some(json!({
                    "artifact_id": enriched_artifact.artifact_id,
                    "artifact_type": enriched_artifact.metadata.artifact_type,
                    "source": "a2a_agent",
                    "task_id": task_id.as_str(),
                    "context_id": context_id.as_str(),
                })),
            )
            .await
            .ok();

        self.artifact_repo
            .create_artifact(task_id, context_id, &enriched_artifact)
            .await
            .map_err(|e| anyhow!("Failed to persist artifact: {}", e))?;

        self.logger
            .info(
                "artifact_publishing",
                &format!(
                    "Artifact {} persisted to database",
                    enriched_artifact.artifact_id
                ),
            )
            .await
            .ok();

        Ok(())
    }

    pub async fn publish_from_mcp(
        &self,
        artifact: &Artifact,
        task_id: &TaskId,
        context_id: &ContextId,
        tool_name: &str,
        tool_args: &serde_json::Value,
        request_context: &RequestContext,
        call_source: CallSource,
    ) -> Result<()> {
        let enriched_artifact = self.enrich_artifact_with_skill(artifact).await;

        self.logger
            .log(
                LogLevel::Info,
                "artifact_publishing",
                "Publishing artifact from direct MCP tool execution",
                Some(json!({
                    "artifact_id": enriched_artifact.artifact_id,
                    "artifact_type": enriched_artifact.metadata.artifact_type,
                    "source": "mcp_direct_call",
                    "tool_name": tool_name,
                    "task_id": task_id.as_str(),
                    "context_id": context_id.as_str(),
                })),
            )
            .await
            .ok();

        self.artifact_repo
            .create_artifact(task_id, context_id, &enriched_artifact)
            .await
            .map_err(|e| anyhow!("Failed to persist artifact: {}", e))?;

        self.logger
            .info(
                "artifact_publishing",
                &format!(
                    "Artifact {} persisted to database",
                    enriched_artifact.artifact_id
                ),
            )
            .await
            .ok();

        if call_source == CallSource::Direct {
            self.logger
                .info(
                    "artifact_publishing",
                    "Creating technical messages for direct MCP call",
                )
                .await
                .ok();

            let (user_message_id, _seq) = self
                .message_service
                .create_tool_execution_message(
                    task_id,
                    context_id,
                    tool_name,
                    tool_args,
                    request_context,
                )
                .await?;

            self.logger
                .info(
                    "artifact_publishing",
                    &format!(
                        "Created synthetic user message {} for MCP tool {}",
                        user_message_id, tool_name
                    ),
                )
                .await
                .ok();

            let agent_message = Message {
                role: "agent".to_string(),
                message_id: Uuid::new_v4().to_string(),
                task_id: Some(task_id.clone()),
                context_id: context_id.clone(),
                kind: "message".to_string(),
                parts: vec![Part::Text(TextPart {
                    text: format!(
                        "Tool execution completed successfully.\n\nCreated artifact: {} (type: {})",
                        enriched_artifact.artifact_id, enriched_artifact.metadata.artifact_type
                    ),
                })],
                metadata: Some(json!({
                    "source": "mcp_direct_call_response",
                    "tool_name": tool_name,
                    "artifact_id": enriched_artifact.artifact_id,
                    "artifact_type": enriched_artifact.metadata.artifact_type,
                })),
                extensions: None,
                reference_task_ids: None,
            };

            self.message_service
                .persist_messages(
                    task_id,
                    context_id,
                    vec![agent_message],
                    Some(request_context.user_id().as_str()),
                    request_context.session_id().as_str(),
                    request_context.trace_id().as_str(),
                )
                .await?;

            self.logger
                .info(
                    "artifact_publishing",
                    "Created agent response message with artifact reference",
                )
                .await
                .ok();
        } else {
            self.logger
                .info(
                    "artifact_publishing",
                    "Skipping message creation for agentic tool call (AI will synthesize response)",
                )
                .await
                .ok();
        }

        Ok(())
    }
}
