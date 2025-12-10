use anyhow::{anyhow, Result};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::models::a2a::{Artifact, Message, Part, Task, TaskState, TaskStatus, TextPart};
use crate::models::AgentRuntimeInfo;
use crate::repository::{ContextRepository, ExecutionStepRepository, TaskRepository};
use crate::services::a2a_server::builders::task::build_completed_task;
use crate::services::{ArtifactPublishingService, ContextService, SkillService};
use systemprompt_core_ai::{AiMessage, AiService, CallToolResult, MessageRole, ToolCall};
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::{BroadcastClient, RequestContext};
use systemprompt_identifiers::{AgentName, SessionId, TraceId, UserId};
use systemprompt_models::execution::{
    EventMessage, EventMessagePart, EventTask, EventTaskStatus, TaskCreatedPayload,
};
use systemprompt_models::TaskMetadata;

use super::artifact::ArtifactBuilder;
use super::strategies::{ExecutionContext, ExecutionStrategySelector};

#[derive(Debug)]
pub enum StreamEvent {
    Text(String),
    ToolCallStarted(ToolCall),
    ToolResult {
        call_id: String,
        result: CallToolResult,
    },
    ArtifactUpdate {
        artifact: Artifact,
        append: bool,
        last_chunk: bool,
    },
    ExecutionStepUpdate {
        step: crate::models::ExecutionStep,
    },
    Complete {
        full_text: String,
        artifacts: Vec<Artifact>,
    },
    Error(String),
}

pub struct MessageProcessor {
    db_pool: DbPool,
    ai_service: Arc<AiService>,
    log: LogService,
    task_repo: TaskRepository,
    context_repo: ContextRepository,
    context_service: ContextService,
    skill_service: Arc<SkillService>,
    execution_step_repo: Arc<ExecutionStepRepository>,
}

impl std::fmt::Debug for MessageProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageProcessor")
            .field("ai_service", &"<Arc<AiService>>")
            .field("log", &"<LogService>")
            .finish()
    }
}

impl MessageProcessor {
    pub fn new(
        db_pool: DbPool,
        ai_service: Arc<AiService>,
        log: LogService,
        broadcaster: Arc<dyn BroadcastClient>,
    ) -> Self {
        let task_repo = TaskRepository::new(db_pool.clone());
        let context_repo = ContextRepository::new(db_pool.clone());
        let context_service = ContextService::new(db_pool.clone());
        let skill_service = Arc::new(SkillService::new(db_pool.clone(), broadcaster.clone()));
        let execution_step_repo = Arc::new(ExecutionStepRepository::new(db_pool.clone()));

        Self {
            db_pool,
            ai_service,
            log,
            task_repo,
            context_repo,
            context_service,
            skill_service,
            execution_step_repo,
        }
    }

    pub async fn load_agent_runtime(&self, agent_name: &str) -> Result<AgentRuntimeInfo> {
        use crate::services::registry::AgentRegistry;

        let registry = AgentRegistry::new().await?;
        let agent_config = registry
            .get_agent(agent_name)
            .await
            .map_err(|_| anyhow!("Agent not found"))?;

        Ok(agent_config.into())
    }

    pub async fn handle_message(
        &self,
        message: Message,
        agent_name: &str,
        context: &RequestContext,
    ) -> Result<Task> {
        let _ = self
            .log
            .info(
                "message_processor",
                &format!("Handling non-streaming message for agent: {agent_name}"),
            )
            .await;

        let agent_runtime = self.load_agent_runtime(agent_name).await?;

        // Explicitly validate that the context exists and the user owns it
        self.context_repo
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

        self.log
            .info(
                "message_processor",
                &format!(
                    "Context validated for context_id: {}, user_id: {}",
                    message.context_id,
                    context.user_id()
                ),
            )
            .await
            .ok();

        // A2A Spec: taskId is optional - when absent, this is a NEW task, when present,
        // CONTINUING existing task
        let task_id = match message.task_id.clone() {
            Some(existing_task_id) => {
                self.log
                    .info(
                        "message_processor",
                        &format!("Continuing existing task: {existing_task_id}"),
                    )
                    .await
                    .ok();
                existing_task_id
            },
            None => {
                let new_task_id = systemprompt_identifiers::TaskId::new(Uuid::new_v4().to_string());
                self.log
                    .info(
                        "message_processor",
                        &format!("Starting NEW task with generated ID: {new_task_id}"),
                    )
                    .await
                    .ok();
                new_task_id
            },
        };

        // Persist task to database immediately (matches streaming flow behavior)
        let metadata = TaskMetadata::new_agent_message(agent_name.to_string());

        let task = Task {
            id: task_id.clone(),
            context_id: message.context_id.clone(),
            status: TaskStatus {
                state: TaskState::Submitted,
                message: None,
                timestamp: Some(chrono::Utc::now()),
            },
            history: None,
            artifacts: None,
            metadata: Some(metadata),
            kind: "task".to_string(),
        };

        if let Err(e) = self
            .task_repo
            .create_task(
                &task,
                &UserId::new(context.user_id().as_str()),
                &SessionId::new(context.session_id().as_str()),
                &TraceId::new(context.trace_id().as_str()),
                agent_name,
            )
            .await
        {
            return Err(anyhow!("Failed to persist task at start: {}", e));
        }

        self.log
            .info(
                "message_processor",
                &format!("Task {} persisted to database", task_id),
            )
            .await
            .ok();

        self.broadcast_task_created(&task_id, &message.context_id, context, &message, agent_name)
            .await;

        // Mark task as working before processing starts (sets started_at timestamp)
        let working_timestamp = chrono::Utc::now();
        if let Err(e) = self
            .task_repo
            .update_task_state(task_id.as_str(), TaskState::Working, &working_timestamp)
            .await
        {
            self.log
                .error(
                    "message_processor",
                    &format!("Failed to mark task as working: {e}"),
                )
                .await
                .ok();
        }

        let mut chunk_rx = self
            .process_message_stream(
                &message,
                &agent_runtime,
                agent_name,
                context,
                task_id.clone(),
            )
            .await?;

        let mut response_text = String::new();
        let mut tool_artifacts = Vec::new();

        while let Some(event) = chunk_rx.recv().await {
            match event {
                StreamEvent::Text(text) => {
                    response_text.push_str(&text);
                },
                StreamEvent::ArtifactUpdate { artifact, .. } => {
                    tool_artifacts.push(artifact);
                },
                StreamEvent::Complete {
                    full_text,
                    artifacts,
                } => {
                    response_text = full_text;
                    tool_artifacts = artifacts;
                },
                StreamEvent::Error(error) => {
                    return Err(anyhow!(error));
                },
                _ => {},
            }
        }

        let task = build_completed_task(
            task_id,
            message.context_id.clone(),
            response_text.clone(),
            message.clone(),
            tool_artifacts,
        );

        let agent_message = task.status.message.clone().unwrap_or_else(|| {
            let client_message_id = message
                .metadata
                .as_ref()
                .and_then(|m| m.get("clientMessageId"))
                .cloned();

            let metadata = client_message_id.map(|id| serde_json::json!({"clientMessageId": id}));

            Message {
                role: "agent".to_string(),
                parts: vec![Part::Text(TextPart {
                    text: response_text.clone(),
                })],
                message_id: Uuid::new_v4().to_string(),
                task_id: Some(task.id.clone()),
                context_id: task.context_id.clone(),
                kind: "message".to_string(),
                metadata,
                extensions: None,
                reference_task_ids: None,
            }
        });

        if context.user_type() == systemprompt_models::auth::UserType::Anon {
            self.log
                .warn(
                    "message_processor",
                    &format!(
                        "Saving messages for anonymous user - context: {:?}, session: {}",
                        message.context_id,
                        context.session_id()
                    ),
                )
                .await
                .ok();
        }

        if let Err(e) = self
            .persist_completed_task(&task, &message, &agent_message, context, agent_name, false)
            .await
        {
            self.log
                .error(
                    "message_processor",
                    &format!("Failed to persist completed task: {e}"),
                )
                .await
                .ok();

            let failed_timestamp = chrono::Utc::now();
            if let Err(update_err) = self
                .task_repo
                .update_task_state(task.id.as_str(), TaskState::Failed, &failed_timestamp)
                .await
            {
                self.log
                    .error(
                        "message_processor",
                        &format!("Failed to update task to failed state: {update_err}"),
                    )
                    .await
                    .ok();
            }

            return Err(e);
        }

        Ok(task)
    }

    fn extract_message_text(&self, message: &Message) -> Result<String> {
        for part in &message.parts {
            if let Part::Text(text_part) = part {
                return Ok(text_part.text.clone());
            }
        }
        Err(anyhow!("No text content found in message"))
    }

    async fn broadcast_task_created(
        &self,
        task_id: &systemprompt_identifiers::TaskId,
        context_id: &systemprompt_identifiers::ContextId,
        context: &RequestContext,
        user_message: &Message,
        agent_name: &str,
    ) {
        let event_task = Self::build_event_task(task_id, context_id, user_message, agent_name);
        let task_created_payload = TaskCreatedPayload { task: event_task };

        let api_url = std::env::var("API_INTERNAL_URL")
            .or_else(|_| std::env::var("API_EXTERNAL_URL"))
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        let webhook_url = format!("{}/api/v1/webhook/broadcast", api_url);

        let payload = json!({
            "event_type": "task_created",
            "entity_id": task_id.as_str(),
            "context_id": context_id.as_str(),
            "user_id": context.user_id().as_str(),
            "task_data": serde_json::to_value(&task_created_payload).expect("TaskCreatedPayload serialization failed")
        });

        let token = context.auth.auth_token.as_str();
        let client = reqwest::Client::new();
        match client
            .post(&webhook_url)
            .header("Authorization", format!("Bearer {token}"))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    self.log
                        .info(
                            "message_processor",
                            &format!("Broadcast task_created via webhook for task {task_id}"),
                        )
                        .await
                        .ok();
                } else {
                    self.log
                        .warn(
                            "message_processor",
                            &format!(
                                "Webhook broadcast failed: status={}, task_id={}",
                                response.status(),
                                task_id
                            ),
                        )
                        .await
                        .ok();
                }
            },
            Err(e) => {
                self.log
                    .warn(
                        "message_processor",
                        &format!("Webhook broadcast error: {e}, task_id={task_id}"),
                    )
                    .await
                    .ok();
            },
        }
    }

    pub async fn persist_completed_task(
        &self,
        task: &Task,
        user_message: &Message,
        agent_message: &Message,
        context: &RequestContext,
        _agent_name: &str,
        artifacts_already_published: bool,
    ) -> Result<Task> {
        let updated_task = self
            .task_repo
            .update_task_and_save_messages(
                task,
                user_message,
                agent_message,
                Some(context.user_id().as_str()),
                context.session_id().as_str(),
                context.trace_id().as_str(),
            )
            .await
            .map_err(|e| anyhow!("Failed to update task and save messages: {}", e))?;

        if !artifacts_already_published {
            if let Some(ref artifacts) = task.artifacts {
                let publishing_service =
                    ArtifactPublishingService::new(self.db_pool.clone(), self.log.clone());
                for artifact in artifacts {
                    publishing_service
                        .publish_from_a2a(artifact, &task.id, &task.context_id, &context.user_id())
                        .await
                        .map_err(|e| {
                            anyhow!("Failed to publish artifact {}: {}", artifact.artifact_id, e)
                        })?;
                }

                self.log
                    .info(
                        "message_processor",
                        &format!(
                            "Published {} artifacts for task {}",
                            artifacts.len(),
                            task.id
                        ),
                    )
                    .await
                    .ok();
            }
        }

        self.log
            .info(
                "message_processor",
                &format!(
                    "Persisted task {} for context {:?} with user_id: {:?}",
                    task.id,
                    task.context_id,
                    context.user_id()
                ),
            )
            .await
            .ok();

        Ok(updated_task)
    }

    pub async fn process_message_stream(
        &self,
        a2a_message: &Message,
        agent_runtime: &AgentRuntimeInfo,
        agent_name: &str,
        context: &RequestContext,
        task_id: systemprompt_identifiers::TaskId,
    ) -> Result<mpsc::UnboundedReceiver<StreamEvent>> {
        let (tx, rx) = mpsc::unbounded_channel();

        let ai_service = self.ai_service.clone();
        let agent_runtime = agent_runtime.clone();
        let agent_name_string = agent_name.to_string();
        let agent_name_typed = AgentName::new(agent_name);
        let log = self.log.clone();
        let user_text = self.extract_message_text(a2a_message)?;

        let context_id = &a2a_message.context_id;
        let conversation_history = self
            .context_service
            .load_conversation_history(context_id.as_str())
            .await
            .unwrap_or_default();
        self.log
            .info(
                "message_processor",
                &format!(
                    "Loaded {} historical messages for context {}",
                    conversation_history.len(),
                    context_id
                ),
            )
            .await
            .ok();

        let _a2a_message = a2a_message.clone();
        let context_id_str = context_id.as_str().to_string();
        let context_id_owned = context_id.clone(); // Clone for use inside spawn
        let task_id_str = task_id.as_str().to_string();

        let request_ctx = context.clone().with_task_id(task_id.clone());
        let db_pool = self.db_pool.clone();
        let _auth_token_for_artifacts = context.auth_token().clone();
        let skill_service = self.skill_service.clone();
        let execution_step_repo = self.execution_step_repo.clone();

        tokio::spawn(async move {
            log.info(
                "message_processor",
                &format!(
                    "Processing streaming message for agent {} with {} history messages",
                    agent_name_string,
                    conversation_history.len()
                ),
            )
            .await
            .ok();

            let mut ai_messages = Vec::new();

            if !agent_runtime.skills.is_empty() {
                log.info(
                    "message_processor",
                    &format!(
                        "Loading {} skills for agent: {:?}",
                        agent_runtime.skills.len(),
                        agent_runtime.skills
                    ),
                )
                .await
                .ok();

                let mut skills_prompt = String::from(
                    "# Your Skills\n\nYou have the following skills that define your capabilities \
                     and writing style:\n\n",
                );

                for skill_id in &agent_runtime.skills {
                    match skill_service.load_skill(skill_id, &request_ctx).await {
                        Ok(skill_content) => {
                            log.info(
                                "message_processor",
                                &format!(
                                    "Loaded skill '{}' ({} chars)",
                                    skill_id,
                                    skill_content.len()
                                ),
                            )
                            .await
                            .ok();
                            skills_prompt.push_str(&format!(
                                "## {} Skill\n\n{}\n\n---\n\n",
                                skill_id, skill_content
                            ));
                        },
                        Err(e) => {
                            log.warn(
                                "message_processor",
                                &format!("Failed to load skill '{skill_id}': {e}"),
                            )
                            .await
                            .ok();
                        },
                    }
                }

                ai_messages.push(AiMessage {
                    role: MessageRole::System,
                    content: skills_prompt,
                });

                log.info("message_processor", "Skills injected into agent context")
                    .await
                    .ok();
            }

            if let Some(system_prompt) = &agent_runtime.system_prompt {
                ai_messages.push(AiMessage {
                    role: MessageRole::System,
                    content: system_prompt.clone(),
                });
            }

            ai_messages.extend(conversation_history);

            ai_messages.push(AiMessage {
                role: MessageRole::User,
                content: user_text,
            });

            let ai_messages_for_synthesis = ai_messages.clone();

            let has_tools = !agent_runtime.mcp_servers.is_empty();
            log.info(
                "message_processor",
                &format!(
                    "Agent has {} MCP servers, has_tools: {}",
                    agent_runtime.mcp_servers.len(),
                    has_tools
                ),
            )
            .await
            .ok();

            let ai_service_for_builder = ai_service.clone();

            let selector = ExecutionStrategySelector::new();
            let strategy = selector.select_strategy(has_tools);

            let execution_context = ExecutionContext {
                ai_service: ai_service.clone(),
                skill_service: skill_service.clone(),
                agent_runtime: agent_runtime.clone(),
                agent_name: agent_name_typed.clone(),
                task_id: task_id.clone(),
                context_id: context_id_owned,
                tx: tx.clone(),
                log: log.clone(),
                request_ctx: request_ctx.clone(),
                execution_step_repo: execution_step_repo.clone(),
            };

            let execution_result = match strategy
                .execute(execution_context, ai_messages.clone())
                .await
            {
                Ok(result) => result,
                Err(e) => {
                    log.error("message_processor", &format!("Execution failed: {e}"))
                        .await
                        .ok();

                    let tracking =
                        crate::services::ExecutionTrackingService::new(execution_step_repo.clone());
                    if let Err(fail_err) = tracking
                        .fail_in_progress_steps(&task_id.as_str(), &e.to_string())
                        .await
                    {
                        log.error(
                            "message_processor",
                            &format!("Failed to mark steps as failed: {fail_err}"),
                        )
                        .await
                        .ok();
                    }

                    tx.send(StreamEvent::Error(format!("Execution failed: {e}")))
                        .ok();
                    return;
                },
            };

            let (accumulated_text, tool_calls, tool_results, _iterations) = (
                execution_result.accumulated_text,
                execution_result.tool_calls,
                execution_result.tool_results,
                execution_result.iterations,
            );

            log.info(
                "message_processor",
                &format!(
                    "Processing complete - text_len: {}, tool_calls: {}, tool_results: {}",
                    accumulated_text.len(),
                    tool_calls.len(),
                    tool_results.len()
                ),
            )
            .await
            .ok();

            log.info(
                "message_processor",
                &format!(
                    "Building artifacts - tool_results: {}, tool_calls: {}",
                    tool_results.len(),
                    tool_calls.len()
                ),
            )
            .await
            .ok();

            let artifacts: Result<Vec<Artifact>, anyhow::Error> = if !tool_results.is_empty() {
                let has_structured_content =
                    tool_results.iter().any(|r| r.structured_content.is_some());

                if has_structured_content {
                    log.info(
                        "message_processor",
                        "Tool results contain structured_content - building A2A artifacts from \
                         agentic MCP calls",
                    )
                    .await
                    .ok();

                    let tool_provider = Arc::new(super::artifact::AiServiceToolProvider::new(
                        ai_service_for_builder.clone(),
                    ));
                    let execution_lookup = Arc::new(
                        super::artifact::DatabaseExecutionIdLookup::new(db_pool.clone()),
                    );

                    let artifact_builder = ArtifactBuilder::new(
                        tool_calls.clone(),
                        tool_results.clone(),
                        tool_provider,
                        execution_lookup,
                        context_id_str.clone(),
                        task_id_str.clone(),
                        request_ctx.clone(),
                        log.clone(),
                    );

                    artifact_builder
                        .build_artifacts(&agent_name_string)
                        .await
                        .map_err(|e| anyhow!("Failed to build artifacts: {}", e))
                } else {
                    log.info(
                        "message_processor",
                        "No structured_content - ephemeral tool calls, skipping A2A artifact \
                         building",
                    )
                    .await
                    .ok();
                    Ok(Vec::new())
                }
            } else {
                log.info(
                    "message_processor",
                    "No tool_results - no artifacts expected",
                )
                .await
                .ok();
                Ok(Vec::new())
            };

            let artifacts = match artifacts {
                Ok(arts) => arts,
                Err(e) => {
                    log.error(
                        "message_processor",
                        &format!("Failed to build artifacts: {e}"),
                    )
                    .await
                    .ok();
                    Vec::new()
                },
            };

            let final_text = if !tool_calls.is_empty() && !tool_results.is_empty() {
                log.info(
                    "message_processor",
                    &format!(
                        "Synthesizing results from {} tool calls with {} artifacts",
                        tool_calls.len(),
                        artifacts.len()
                    ),
                )
                .await
                .ok();

                match super::ai_executor::synthesize_tool_results_with_artifacts(
                    ai_service_for_builder.clone(),
                    &agent_runtime,
                    ai_messages_for_synthesis.clone(),
                    &accumulated_text,
                    &tool_calls,
                    &tool_results,
                    &artifacts,
                    tx.clone(),
                    &log,
                    request_ctx.clone(),
                    skill_service.clone(),
                )
                .await
                {
                    Ok(synthesized) => synthesized,
                    Err(_) => {
                        log.warn(
                            "message_processor",
                            "Synthesis failed, using initial response",
                        )
                        .await
                        .ok();
                        accumulated_text.clone()
                    },
                }
            } else {
                if tool_calls.is_empty() && !accumulated_text.is_empty() {
                    log.warn(
                        "message_processor",
                        &format!(
                            "Synthesis skipped: Agent produced text without tool calls, response \
                             length: {} chars",
                            accumulated_text.len()
                        ),
                    )
                    .await
                    .ok();
                }
                accumulated_text.clone()
            };

            log.info(
                "message_processor",
                &format!("Sending Complete event with {} artifacts", artifacts.len()),
            )
            .await
            .ok();

            for (idx, artifact) in artifacts.iter().enumerate() {
                log.info(
                    "message_processor",
                    &format!(
                        "Complete artifact {}/{}: id={}",
                        idx + 1,
                        artifacts.len(),
                        artifact.artifact_id
                    ),
                )
                .await
                .ok();
            }

            let send_result = tx.send(StreamEvent::Complete {
                full_text: final_text.clone(),
                artifacts: artifacts.clone(),
            });

            if send_result.is_err() {
                log.error(
                    "message_processor",
                    "Failed to send Complete event, channel closed",
                )
                .await
                .ok();
            } else {
                log.info(
                    "message_processor",
                    &format!("Sent Complete event with {} artifacts", artifacts.len()),
                )
                .await
                .ok();
            }
        });

        Ok(rx)
    }

    fn build_event_task(
        task_id: &systemprompt_identifiers::TaskId,
        context_id: &systemprompt_identifiers::ContextId,
        user_message: &Message,
        agent_name: &str,
    ) -> EventTask {
        let event_message = EventMessage {
            role: user_message.role.to_lowercase(),
            parts: user_message
                .parts
                .iter()
                .map(|part| match part {
                    Part::Text(text_part) => EventMessagePart::Text {
                        text: text_part.text.clone(),
                    },
                    Part::Data(data_part) => EventMessagePart::Data {
                        data: serde_json::Value::Object(data_part.data.clone()),
                    },
                    Part::File(file_part) => EventMessagePart::File {
                        file: serde_json::to_value(&file_part.file).unwrap_or_default(),
                    },
                })
                .collect(),
            message_id: user_message.message_id.clone(),
        };

        EventTask {
            id: task_id.clone(),
            context_id: context_id.clone(),
            status: EventTaskStatus {
                state: "submitted".to_string(),
                message: None,
                timestamp: Some(chrono::Utc::now()),
            },
            history: Some(vec![event_message]),
            artifacts: None,
            metadata: Some(TaskMetadata::new_agent_message(agent_name.to_string())),
            kind: "task".to_string(),
        }
    }
}
