use std::sync::Arc;

use axum::response::sse::Event;
use serde_json::json;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::RequestContext;
use systemprompt_identifiers::{ContextId, SessionId, TaskId, TraceId, UserId};
use systemprompt_models::execution::{
    EventMessage, EventMessagePart, EventTask, EventTaskStatus, TaskCreatedPayload,
};
use systemprompt_models::TaskMetadata;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use crate::models::a2a::protocol::PushNotificationConfig;
use crate::models::a2a::{Message, Part, Task, TaskState, TaskStatus};
use crate::repository::{ContextRepository, PushNotificationConfigRepository, TaskRepository};
use crate::services::a2a_server::errors::classify_database_error;
use crate::services::a2a_server::handlers::AgentHandlerState;

pub async fn detect_mcp_server_and_update_context(
    agent_name: &str,
    context: &mut RequestContext,
    log: &LogService,
) {
    use systemprompt_core_mcp::services::registry::McpServerRegistry;

    let is_mcp_server = match McpServerRegistry::new().await {
        Ok(registry) => registry
            .get_server_by_name(agent_name)
            .await
            .ok()
            .flatten()
            .is_some(),
        Err(_) => false,
    };

    if is_mcp_server && context.agent_name().as_str() != agent_name {
        log.info(
            "sse_init",
            &format!(
                "MCP server '{}' handling request from agent '{}'",
                agent_name,
                context.agent_name().as_str()
            ),
        )
        .await
        .ok();
    } else if !is_mcp_server && context.agent_name().as_str() != agent_name {
        log.warn(
            "sse_init",
            &format!(
                "Agent mismatch: context='{}', service='{}'. Using service name.",
                context.agent_name().as_str(),
                agent_name
            ),
        )
        .await
        .ok();

        use systemprompt_identifiers::AgentName;
        context.execution.agent_name = AgentName::new(agent_name.to_string());
    }
}

pub fn resolve_task_id(message: &Message) -> TaskId {
    if let Some(existing_task_id) = message.task_id.clone() {
        existing_task_id
    } else {
        TaskId::new(Uuid::new_v4().to_string())
    }
}

pub async fn validate_context(
    context_id: &ContextId,
    user_id: &str,
    state: &Arc<AgentHandlerState>,
    tx: &UnboundedSender<Event>,
    request_id: &Option<serde_json::Value>,
    log: &LogService,
) -> Result<(), ()> {
    let context_repo = ContextRepository::new(state.db_pool.clone());

    if let Err(e) = context_repo.get_context(context_id.as_str(), user_id).await {
        log.error(
            "sse_init",
            &format!(
                "Context validation failed - context_id: {}, user_id: {}, error: {}",
                context_id, user_id, e
            ),
        )
        .await
        .ok();

        let error_event = json!({
            "jsonrpc": "2.0",
            "error": {
                "code": -32603,
                "message": format!("Context validation failed: {e}")
            },
            "id": request_id
        });
        let _ = tx.send(Event::default().data(error_event.to_string()));
        return Err(());
    }

    log.info(
        "sse_init",
        &format!(
            "Context validated for context_id: {}, user_id: {}",
            context_id, user_id
        ),
    )
    .await
    .ok();

    Ok(())
}

pub async fn persist_initial_task(
    task_id: &TaskId,
    context_id: &ContextId,
    agent_name: &str,
    context: &RequestContext,
    state: &Arc<AgentHandlerState>,
    tx: &UnboundedSender<Event>,
    request_id: &Option<serde_json::Value>,
    log: &LogService,
) -> Result<TaskRepository, ()> {
    let task_repo = TaskRepository::new(state.db_pool.clone());
    let metadata = TaskMetadata::new_agent_message(agent_name.to_string());

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
            agent_name,
        )
        .await
    {
        log.error(
            "sse_init",
            &format!("Failed to persist task at start: {e}"),
        )
        .await
        .ok();

        let error_detail = classify_database_error(&e);
        let error_event = json!({
            "jsonrpc": "2.0",
            "error": {
                "code": -32603,
                "message": format!("Failed to create task: {error_detail}")
            },
            "id": request_id
        });
        let _ = tx.send(Event::default().data(error_event.to_string()));
        return Err(());
    }

    log.info(
        "sse_init",
        &format!("Task {} persisted to database at stream start", task_id),
    )
    .await
    .ok();

    if let Err(e) = task_repo
        .track_agent_in_context(context_id.as_str(), agent_name)
        .await
    {
        log.warn(
            "sse_init",
            &format!("Failed to track agent in context: {e}"),
        )
        .await
        .ok();
    }

    Ok(task_repo)
}

pub async fn save_push_notification_config(
    task_id: &TaskId,
    callback_config: &Option<PushNotificationConfig>,
    state: &Arc<AgentHandlerState>,
    log: &LogService,
) {
    if let Some(ref config) = callback_config {
        log.info(
            "sse_init",
            &format!("Push notification callback registered: {}", config.url),
        )
        .await
        .ok();

        let config_repo = PushNotificationConfigRepository::new(state.db_pool.clone());

        if let Err(e) = config_repo.add_config(task_id.as_str(), config).await {
            log.warn(
                "sse_init",
                &format!("Failed to save inline push notification config: {e}"),
            )
            .await
            .ok();
        } else {
            log.info(
                "sse_init",
                &format!("Push notification config saved for task {task_id}"),
            )
            .await
            .ok();
        }
    }
}

pub fn emit_start_event(
    tx: &UnboundedSender<Event>,
    task_id: &TaskId,
    context_id: &ContextId,
    request_id: &Option<serde_json::Value>,
) {
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
}

pub fn emit_message_received_event(
    tx: &UnboundedSender<Event>,
    task_id: &TaskId,
    context_id: &ContextId,
    request_id: &Option<serde_json::Value>,
) {
    let event = json!({
        "jsonrpc": "2.0",
        "result": {
            "kind": "message.received",
            "taskId": task_id,
            "contextId": context_id,
            "timestamp": chrono::Utc::now().to_rfc3339()
        },
        "id": request_id
    });

    let _ = tx.send(Event::default().data(event.to_string()));
}

pub async fn broadcast_task_created(
    task_id: &TaskId,
    context_id: &ContextId,
    user_id: &str,
    user_message: &Message,
    agent_name: &str,
    token: &str,
    log: &LogService,
) {
    let event_task = build_event_task(task_id, context_id, user_message, agent_name);
    let task_created_payload = TaskCreatedPayload { task: event_task };

    let api_url = std::env::var("API_INTERNAL_URL")
        .or_else(|_| std::env::var("API_EXTERNAL_URL"))
        .unwrap_or_else(|_| "http://localhost:8080".to_string());

    let webhook_url = format!("{}/api/v1/webhook/broadcast", api_url);

    let payload = json!({
        "event_type": "task_created",
        "entity_id": task_id.as_str(),
        "context_id": context_id.as_str(),
        "user_id": user_id,
        "task_data": serde_json::to_value(&task_created_payload).expect("TaskCreatedPayload serialization failed")
    });

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
                log.info(
                    "sse_init",
                    &format!("Broadcast task_created via webhook for task {task_id}"),
                )
                .await
                .ok();
            } else {
                log.warn(
                    "sse_init",
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
            log.warn(
                "sse_init",
                &format!("Webhook broadcast error: {e}, task_id={task_id}"),
            )
            .await
            .ok();
        },
    }
}

pub async fn broadcast_task_completed(task: &Task, user_id: &str, token: &str, log: &LogService) {
    let api_url = std::env::var("API_INTERNAL_URL")
        .or_else(|_| std::env::var("API_EXTERNAL_URL"))
        .unwrap_or_else(|_| "http://localhost:8080".to_string());

    let webhook_url = format!("{}/api/v1/webhook/broadcast", api_url);

    let payload = json!({
        "event_type": "task_completed",
        "entity_id": task.id.as_str(),
        "context_id": task.context_id.as_str(),
        "user_id": user_id,
        "task_data": serde_json::to_value(task).expect("Task serialization failed")
    });

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
                log.info(
                    "sse_complete",
                    &format!("Broadcast task_completed for task {}", task.id),
                )
                .await
                .ok();
            } else {
                log.warn(
                    "sse_complete",
                    &format!(
                        "Webhook failed: status={}, task_id={}",
                        response.status(),
                        task.id
                    ),
                )
                .await
                .ok();
            }
        },
        Err(e) => {
            log.warn(
                "sse_complete",
                &format!("Webhook error: {}, task_id={}", e, task.id),
            )
            .await
            .ok();
        },
    }
}

fn build_event_task(
    task_id: &TaskId,
    context_id: &ContextId,
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

pub async fn broadcast_execution_step(
    step: &systemprompt_models::ExecutionStep,
    task_id: &TaskId,
    context_id: &ContextId,
    user_id: &str,
    token: &str,
    log: &LogService,
) {
    let api_url = std::env::var("API_INTERNAL_URL")
        .or_else(|_| std::env::var("API_EXTERNAL_URL"))
        .unwrap_or_else(|_| "http://localhost:8080".to_string());

    let webhook_url = format!("{}/api/v1/webhook/broadcast", api_url);

    let step_data = serde_json::to_value(step).unwrap_or_default();

    let payload = json!({
        "event_type": "execution_step",
        "entity_id": step.step_id.as_str(),
        "context_id": context_id.as_str(),
        "user_id": user_id,
        "step_data": step_data
    });

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
                log.debug(
                    "sse_step",
                    &format!(
                        "Broadcast execution_step via webhook | step_id={}, task_id={}",
                        step.step_id, task_id
                    ),
                )
                .await
                .ok();
            } else {
                log.warn(
                    "sse_step",
                    &format!(
                        "Webhook broadcast failed: status={}, step_id={}",
                        response.status(),
                        step.step_id
                    ),
                )
                .await
                .ok();
            }
        },
        Err(e) => {
            log.warn(
                "sse_step",
                &format!("Webhook broadcast error: {}, step_id={}", e, step.step_id),
            )
            .await
            .ok();
        },
    }
}
