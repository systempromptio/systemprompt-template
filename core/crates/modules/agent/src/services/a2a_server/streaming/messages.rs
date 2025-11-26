use axum::response::sse::Event;
use serde_json::json;
use std::sync::Arc;
use systemprompt_traits::validation::Validate;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;

use crate::models::a2a::protocol::PushNotificationConfig;
use crate::models::a2a::{Message, Part, Task, TaskState, TaskStatus, TextPart};
use crate::models::AgentRuntimeInfo;
use crate::services::a2a_server::errors::classify_database_error;
use crate::services::a2a_server::handlers::AgentHandlerState;
use crate::services::a2a_server::processing::message::MessageProcessor;
use systemprompt_core_system::RequestContext;
use systemprompt_identifiers::{SessionId, TraceId, UserId};
use systemprompt_models::TaskMetadata;

pub async fn create_sse_stream(
    message: Message,
    agent_name: String,
    state: Arc<AgentHandlerState>,
    request_id: Option<serde_json::Value>,
    context: RequestContext,
    callback_config: Option<PushNotificationConfig>,
) -> UnboundedReceiverStream<Event> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let log = state.log.clone();

    log.info(
        "sse_messages",
        "🔵 create_sse_stream() called - spawning tokio task",
    )
    .await
    .ok();

    tokio::spawn(async move {
        let tx = tx;
        eprintln!("===== TOKIO TASK STARTED: FIRST LINE OF SPAWNED TASK =====");
        log.info(
            "sse_messages",
            "🟢 Inside tokio::spawn - task execution started",
        )
        .await
        .ok();
        eprintln!("===== TOKIO TASK: After first log =====");

        // Check if the service is an MCP server (tool provider vs conversational agent)
        let mut context = context;
        use systemprompt_core_mcp::services::registry::McpServerRegistry;

        let is_mcp_server = match McpServerRegistry::new().await {
            Ok(registry) => registry
                .get_server_by_name(&agent_name)
                .await
                .ok()
                .flatten()
                .is_some(),
            Err(_) => false,
        };

        if is_mcp_server && context.agent_name().as_str() != agent_name.as_str() {
            // MCP server handling proxied request - preserve calling agent's name
            log.info(
                "sse_messages",
                &format!(
                    "MCP server '{}' handling request from agent '{}'",
                    agent_name,
                    context.agent_name().as_str()
                ),
            )
            .await
            .ok();
        } else if !is_mcp_server && context.agent_name().as_str() != agent_name.as_str() {
            // Agent-to-agent mismatch - use service name
            log.warn(
                "sse_messages",
                &format!(
                    "Agent mismatch: context='{}', service='{}'. Using service name.",
                    context.agent_name().as_str(),
                    agent_name
                ),
            )
            .await
            .ok();

            use systemprompt_identifiers::AgentName;
            context.execution.agent_name = AgentName::new(agent_name.clone());
        }

        // A2A Spec: taskId is optional - when absent, this is a NEW task, when present, CONTINUING existing task
        let task_id = if let Some(existing_task_id) = message.task_id.clone() {
            log.info(
                "sse_messages",
                &format!("Continuing existing task: {}", existing_task_id),
            )
            .await
            .ok();
            existing_task_id
        } else {
            let new_task_id = systemprompt_identifiers::TaskId::new(Uuid::new_v4().to_string());
            log.info(
                "sse_messages",
                &format!("Starting NEW task with generated ID: {}", new_task_id),
            )
            .await
            .ok();
            new_task_id
        };

        let context_id = message.context_id.clone();
        let message_id = Uuid::new_v4().to_string();

        log.info(
            "sse_messages",
            &format!(
                "Generated IDs: task_id={}, context_id={}, message_id={}",
                task_id, context_id, message_id
            ),
        )
        .await
        .ok();

        // Validate that the context exists and the user owns it before creating a task
        use crate::repository::{ArtifactRepository, ContextRepository, TaskRepository};
        let task_repo = Arc::new(TaskRepository::new(state.db_pool.clone()));
        let artifact_repo = Arc::new(ArtifactRepository::new(state.db_pool.clone()));
        let context_repo = ContextRepository::new(state.db_pool.clone(), task_repo, artifact_repo);
        if let Err(e) = context_repo
            .get_context(context_id.as_str(), context.user_id().as_str())
            .await
        {
            log.error(
                "sse_messages",
                &format!(
                    "Context validation failed - context_id: {}, user_id: {}, error: {}",
                    context_id,
                    context.user_id(),
                    e
                ),
            )
            .await
            .ok();

            let error_event = json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32603,
                    "message": format!("Context validation failed: {}", e)
                },
                "id": request_id
            });
            let _ = tx.send(Event::default().data(error_event.to_string()));
            drop(tx);
            return;
        }

        log.info(
            "sse_messages",
            &format!(
                "✓ Context validated for context_id: {}, user_id: {}",
                context_id,
                context.user_id()
            ),
        )
        .await
        .ok();

        // Persist task to database immediately at stream start (A2A spec-compliant)
        let task_repo = TaskRepository::new(state.db_pool.clone());

        let metadata = TaskMetadata::new_agent_message(agent_name.clone());

        let task = Task {
            id: task_id.clone(),
            context_id: context_id.clone(),
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

        if let Err(e) = task_repo
            .create_task(
                &task,
                &UserId::new(context.user_id().as_str()),
                &SessionId::new(context.session_id().as_str()),
                &TraceId::new(context.trace_id().as_str()),
                &agent_name,
            )
            .await
        {
            log.error(
                "sse_messages",
                &format!("Failed to persist task at start: {}", e),
            )
            .await
            .ok();

            let error_detail = classify_database_error(&e);
            let error_event = json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32603,
                    "message": format!("Failed to create task: {}", error_detail)
                },
                "id": request_id
            });
            let _ = tx.send(Event::default().data(error_event.to_string()));
            drop(tx);
            return;
        }

        log.info(
            "sse_messages",
            &format!("✓ Task {} persisted to database at stream start", task_id),
        )
        .await
        .ok();

        if let Err(e) = task_repo
            .track_agent_in_context(context_id.as_str(), &agent_name)
            .await
        {
            log.warn(
                "sse_messages",
                &format!("Failed to track agent in context: {}", e),
            )
            .await
            .ok();
        }

        // Save inline pushNotificationConfig immediately at stream start (A2A spec-compliant)
        if let Some(ref config) = callback_config {
            log.info(
                "sse_messages",
                &format!("Push notification callback registered: {}", config.url),
            )
            .await
            .ok();

            use crate::repository::PushNotificationConfigRepository;
            let config_repo = PushNotificationConfigRepository::new(state.db_pool.clone());

            if let Err(e) = config_repo.add_config(task_id.as_str(), config).await {
                log.warn(
                    "sse_messages",
                    &format!("Failed to save inline push notification config: {}", e),
                )
                .await
                .ok();
            } else {
                log.info(
                    "sse_messages",
                    &format!("✓ Push notification config saved for task {}", task_id),
                )
                .await
                .ok();
            }
        }

        let start_event = json!({
            "jsonrpc": "2.0",
            "result": {
                "kind": "task.started",
                "taskId": task_id,
                "contextId": context_id,
                "timestamp": chrono::Utc::now().to_rfc3339()
            },
            "id": request_id
        });

        let _ = tx.send(Event::default().data(start_event.to_string()));

        use crate::services::registry::AgentRegistry;

        let agent_runtime: AgentRuntimeInfo = match AgentRegistry::new().await {
            Ok(registry) => match registry.get_agent(&agent_name).await {
                Ok(agent_config) => agent_config.into(),
                Err(e) => {
                    log.error(
                        "sse_messages",
                        &format!("Failed to load agent: {} - {}", agent_name, e),
                    )
                    .await
                    .ok();

                    let failed_timestamp = chrono::Utc::now();
                    if let Err(update_err) = task_repo
                        .update_task_state(task_id.as_str(), TaskState::Failed, &failed_timestamp)
                        .await
                    {
                        log.error(
                            "sse_messages",
                            &format!("Failed to update task to failed state: {}", update_err),
                        )
                        .await
                        .ok();
                    }

                    let error_event = json!({
                        "jsonrpc": "2.0",
                        "error": {
                            "code": -32603,
                            "message": "Agent not found"
                        },
                        "id": request_id
                    });
                    let _ = tx.send(Event::default().data(error_event.to_string()));
                    drop(tx);
                    return;
                },
            },
            Err(e) => {
                let error_details = format!("Failed to load agent registry: {} - check if config files exist and services are properly configured", e);
                log.error("sse_messages", &error_details).await.ok();

                let failed_timestamp = chrono::Utc::now();
                if let Err(update_err) = task_repo
                    .update_task_state(task_id.as_str(), TaskState::Failed, &failed_timestamp)
                    .await
                {
                    log.error(
                        "sse_messages",
                        &format!("Failed to update task to failed state: {}", update_err),
                    )
                    .await
                    .ok();
                }

                let error_event = json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32603,
                        "message": "Failed to load agent registry - check system logs for details"
                    },
                    "id": request_id
                });
                let _ = tx.send(Event::default().data(error_event.to_string()));
                drop(tx);
                return;
            },
        };

        let processor = Arc::new(MessageProcessor::new(
            state.db_pool.clone(),
            state.ai_service.clone(),
            log.clone(),
        ));

        log.info(
            "sse_messages",
            &format!(
                "Starting message stream processing for agent: {}",
                agent_name
            ),
        )
        .await
        .ok();

        match processor
            .process_message_stream(
                &message,
                &agent_runtime,
                &agent_name,
                &context,
                task_id.clone(),
            )
            .await
        {
            Ok(mut chunk_rx) => {
                // A2A Spec: Update task to "working" state when processing starts
                let working_timestamp = chrono::Utc::now();
                if let Err(e) = task_repo
                    .update_task_state(task_id.as_str(), TaskState::Working, &working_timestamp)
                    .await
                {
                    log.error(
                        "sse_messages",
                        &format!("Failed to update task to working state: {}", e),
                    )
                    .await
                    .ok();
                } else {
                    log.info(
                        "sse_messages",
                        &format!("✓ Task {} updated to working state", task_id),
                    )
                    .await
                    .ok();

                    // Send TaskStatusUpdateEvent with state="working" (A2A spec-compliant)
                    let working_event = json!({
                        "jsonrpc": "2.0",
                        "result": {
                            "kind": "status-update",
                            "taskId": task_id,
                            "contextId": context_id,
                            "status": {
                                "state": "working",
                                "timestamp": working_timestamp
                            },
                            "final": false
                        },
                        "id": request_id
                    });
                    let _ = tx.send(Event::default().data(working_event.to_string()));
                }

                log.info(
                    "sse_messages",
                    "Stream channel received, waiting for events...",
                )
                .await
                .ok();

                while let Some(event) = chunk_rx.recv().await {
                    match event {
                        crate::services::a2a_server::processing::message::StreamEvent::Text(text) => {
                            let message_event = json!({
                                "jsonrpc": "2.0",
                                "result": {
                                    "kind": "message",
                                    "role": "agent",
                                    "parts": [{
                                        "kind": "text",
                                        "text": text
                                    }],
                                    "messageId": message_id,
                                    "taskId": task_id,
                                    "contextId": context_id,
                                    "timestamp": chrono::Utc::now().to_rfc3339()
                                },
                                "id": request_id
                            });

                            let _ = tx.send(Event::default().data(message_event.to_string()));
                        }
                        crate::services::a2a_server::processing::message::StreamEvent::ToolCallStarted(tool_call) => {
                            let tool_event = json!({
                                "jsonrpc": "2.0",
                                "result": {
                                    "kind": "tool_call",
                                    "id": tool_call.ai_tool_call_id.as_ref(),
                                    "name": tool_call.name,
                                    "arguments": tool_call.arguments
                                },
                                "id": request_id
                            });
                            let _ = tx.send(Event::default().data(tool_event.to_string()));
                        }
                        crate::services::a2a_server::processing::message::StreamEvent::ToolResult { call_id, result } => {
                            use rmcp::model::RawContent;

                            let content_json: Vec<serde_json::Value> = result.content.iter().map(|c| {
                                match &c.raw {
                                    RawContent::Text(text_content) => json!({"type": "text", "text": text_content.text}),
                                    RawContent::Image(image_content) => json!({"type": "image", "data": image_content.data, "mimeType": image_content.mime_type}),
                                    RawContent::ResourceLink(resource) => json!({"type": "resource", "uri": resource.uri, "mimeType": resource.mime_type}),
                                    _ => json!({"type": "unknown"}),
                                }
                            }).collect();

                            // Treat missing is_error flag as error (fail-safe approach)
                            let is_error = if result.is_error.is_none() {
                                log.warn("sse_messages", "Tool result missing is_error flag - treating as error").await.ok();
                                true
                            } else {
                                result.is_error.unwrap()
                            };

                            let result_event = json!({
                                "jsonrpc": "2.0",
                                "result": {
                                    "kind": "tool_result",
                                    "call_id": call_id,
                                    "content": content_json,
                                    "is_error": is_error
                                },
                                "id": request_id
                            });
                            let _ = tx.send(Event::default().data(result_event.to_string()));
                        }
                        crate::services::a2a_server::processing::message::StreamEvent::ArtifactUpdate { artifact, append, last_chunk } => {
                            let artifact_type = &artifact.metadata.artifact_type;

                            artifact.metadata.validate_or_panic();

                            log.info(
                                "sse_stream",
                                &format!(
                                    "📤 PUBLISHING ARTIFACT ID {} TO SSE STREAM (type={}, parts={}, append={}, last={})",
                                    artifact.artifact_id,
                                    artifact_type,
                                    artifact.parts.len(),
                                    append,
                                    last_chunk
                                )
                            ).await.ok();

                            // CRITICAL: Persist artifact to database BEFORE sending SSE event
                            // This ensures FK relationships exist for any artifact parts
                            use crate::services::ArtifactPublishingService;
                            let publishing_service = ArtifactPublishingService::new(state.db_pool.clone(), log.clone());

                            if let Err(e) = publishing_service.publish_from_a2a(
                                &artifact,
                                &task_id,
                                &context_id,
                                &context.user_id(),
                            ).await {
                                log.error(
                                    "sse_stream",
                                    &format!("Failed to persist artifact during streaming: {}", e)
                                ).await.ok();

                                let error_event = json!({
                                    "jsonrpc": "2.0",
                                    "error": {
                                        "code": -32603,
                                        "message": format!("Query execution failed: {}", e)
                                    },
                                    "id": request_id
                                });
                                let _ = tx.send(Event::default().data(error_event.to_string()));
                                break;
                            }

                            log.info(
                                "sse_stream",
                                &format!("✅ Artifact {} persisted to database before SSE emit", artifact.artifact_id)
                            ).await.ok();

                            let artifact_event = json!({
                                "jsonrpc": "2.0",
                                "result": {
                                    "taskId": task_id,
                                    "contextId": context_id,
                                    "kind": "artifact-update",
                                    "artifact": artifact,
                                    "append": append,
                                    "lastChunk": last_chunk
                                },
                                "id": request_id
                            });

                            let _ = tx.send(Event::default().data(artifact_event.to_string()));
                        }
                        crate::services::a2a_server::processing::message::StreamEvent::Complete { full_text, artifacts } => {
                            log.info(
                                "sse_stream",
                                &format!("🎬 RECEIVED Complete event with {} artifacts", artifacts.len()),
                            ).await.ok();

                            for (idx, artifact) in artifacts.iter().enumerate() {
                                log.info(
                                    "sse_stream",
                                    &format!("  📦 Received artifact {}/{}: id={}", idx + 1, artifacts.len(), artifact.artifact_id),
                                ).await.ok();
                            }

                            let artifacts_for_task = if artifacts.is_empty() {
                                log.warn("sse_stream", "⚠️ Artifacts array is EMPTY, setting Task.artifacts to None").await.ok();
                                None
                            } else {
                                log.info("sse_stream", &format!("✅ Setting Task.artifacts to Some({} items)", artifacts.len())).await.ok();
                                Some(artifacts.clone())
                            };

                            let task_metadata = TaskMetadata::new_validated_agent_message(agent_name.clone())
                                .unwrap_or_else(|e| panic!("VALIDATION ERROR creating TaskMetadata: {}", e));

                            let complete_task = Task {
                                id: task_id.clone(),
                                context_id: context_id.clone(),
                                kind: "task".to_string(),
                                status: TaskStatus {
                                    state: TaskState::Completed,
                                    message: Some(Message {
                                        role: "agent".to_string(),
                                        parts: vec![Part::Text(TextPart {
                                            text: full_text.clone(),
                                        })],
                                        message_id: message_id.clone(),
                                        task_id: Some(task_id.clone()),
                                        context_id: context_id.clone(),
                                        kind: "message".to_string(),
                                        metadata: None,
                                        extensions: None,
                                        reference_task_ids: None,
                                    }),
                                    timestamp: Some(chrono::Utc::now()),
                                },
                                history: Some(vec![
                                    message.clone(),
                                    Message {
                                        role: "agent".to_string(),
                                        parts: vec![Part::Text(TextPart {
                                            text: full_text.clone(),
                                        })],
                                        message_id: Uuid::new_v4().to_string(),
                                        task_id: Some(task_id.clone()),
                                        context_id: context_id.clone(),
                                        kind: "message".to_string(),
                                        metadata: None,
                                        extensions: None,
                                        reference_task_ids: None,
                                    }
                                ]),
                                artifacts: artifacts_for_task,
                                metadata: Some(task_metadata),
                            };

                            log.info(
                                "sse_stream",
                                &format!("📋 Task.artifacts = {:?}", if complete_task.artifacts.is_some() { format!("Some({} items)", complete_task.artifacts.as_ref().unwrap().len()) } else { "None".to_string() }),
                            ).await.ok();

                            if let Some(ref metadata) = complete_task.metadata {
                                metadata.validate_or_panic();
                            } else {
                                log.error("sse_stream", "VALIDATION ERROR: Task metadata is None before SSE send").await.ok();
                                panic!("VALIDATION ERROR: Task.metadata cannot be None");
                            }

                            let agent_message = complete_task.status.message.clone().unwrap();

                            match processor.persist_completed_task(
                                &complete_task,
                                &message,
                                &agent_message,
                                &context,
                                &agent_name,
                                true,
                            ).await {
                                Err(e) => {
                                    log.error("sse_messages", &format!("Failed to complete task and persist messages: {}", e)).await.ok();

                                    let failed_timestamp = chrono::Utc::now();
                                    if let Err(update_err) = task_repo.update_task_state(
                                        task_id.as_str(),
                                        TaskState::Failed,
                                        &failed_timestamp,
                                    ).await {
                                        log.error("sse_messages", &format!("Failed to update task to failed state: {}", update_err)).await.ok();
                                    }

                                    let error_event = json!({
                                        "jsonrpc": "2.0",
                                        "error": {
                                            "code": -32603,
                                            "message": format!("Failed to persist task completion: {}", e)
                                        },
                                        "id": request_id
                                    });
                                    let _ = tx.send(Event::default().data(error_event.to_string()));
                                    break;
                                },
                                Ok(task_with_timing) => {
                                    log.info("sse_messages", &format!(
                                        "✅ Task {} completed and persisted with timing: execution_time_ms = {:?}",
                                        task_id,
                                        task_with_timing.metadata.as_ref().and_then(|m| m.execution_time_ms)
                                    )).await.ok();

                                    let complete_event = json!({
                                        "jsonrpc": "2.0",
                                        "result": task_with_timing,
                                        "id": request_id
                                    });

                                    log.info(
                                        "sse_stream",
                                        &format!("🚀 SENDING JSON to frontend: artifacts in result = {:?}",
                                            complete_event.get("result")
                                                .and_then(|r| r.get("artifacts"))
                                                .map(|a| if a.is_null() { "null".to_string() } else if a.is_array() { format!("array[{}]", a.as_array().unwrap().len()) } else { format!("{:?}", a) })
                                        ),
                                    ).await.ok();

                                    let _ = tx.send(Event::default().data(complete_event.to_string()));
                                    break;
                                }
                            }
                        }
                        crate::services::a2a_server::processing::message::StreamEvent::Error(error) => {
                            let failed_timestamp = chrono::Utc::now();
                            if let Err(e) = task_repo.update_task_state(
                                task_id.as_str(),
                                TaskState::Failed,
                                &failed_timestamp,
                            ).await {
                                log.error("sse_messages", &format!("Failed to update task to failed state: {}", e)).await.ok();
                            }

                            let error_event = json!({
                                "jsonrpc": "2.0",
                                "error": {
                                    "code": -32603,
                                    "message": error
                                },
                                "id": request_id
                            });
                            let _ = tx.send(Event::default().data(error_event.to_string()));
                            break;
                        }
                    }
                }
                drop(tx);

                log.info(
                    "sse_messages",
                    "Stream event loop ended - all events processed",
                )
                .await
                .ok();
            },
            Err(e) => {
                log.error(
                    "sse_messages",
                    &format!("Failed to create message stream: {}", e),
                )
                .await
                .ok();

                let failed_timestamp = chrono::Utc::now();
                if let Err(update_err) = task_repo
                    .update_task_state(task_id.as_str(), TaskState::Failed, &failed_timestamp)
                    .await
                {
                    log.error(
                        "sse_messages",
                        &format!("Failed to update task to failed state: {}", update_err),
                    )
                    .await
                    .ok();
                }

                let error_event = json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32603,
                        "message": format!("Failed to process message: {}", e)
                    },
                    "id": request_id
                });
                let _ = tx.send(Event::default().data(error_event.to_string()));
            },
        }
    });

    UnboundedReceiverStream::new(rx)
}
