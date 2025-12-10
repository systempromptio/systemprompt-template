use std::sync::Arc;

use axum::response::sse::Event;
use serde_json::json;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::RequestContext;
use systemprompt_identifiers::{ContextId, TaskId};
use systemprompt_models::TaskMetadata;
use systemprompt_traits::validation::Validate;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use crate::models::a2a::{Artifact, Message, Part, Task, TaskState, TaskStatus, TextPart};
use crate::repository::TaskRepository;
use crate::services::a2a_server::processing::message::MessageProcessor;
use crate::services::a2a_server::streaming::initialization::broadcast_task_completed;

pub enum CompletionResult {
    Break,
}

pub async fn handle_complete(
    tx: &UnboundedSender<Event>,
    full_text: String,
    artifacts: Vec<Artifact>,
    task_id: &TaskId,
    context_id: &ContextId,
    message_id: &str,
    request_id: &Option<serde_json::Value>,
    original_message: &Message,
    agent_name: &str,
    context: &RequestContext,
    token: &str,
    log: &LogService,
    task_repo: &TaskRepository,
    processor: &Arc<MessageProcessor>,
) -> CompletionResult {
    log.info(
        "sse_complete",
        &format!("Received Complete event with {} artifacts", artifacts.len()),
    )
    .await
    .ok();

    for (idx, artifact) in artifacts.iter().enumerate() {
        log.info(
            "sse_complete",
            &format!(
                "Received artifact {}/{}: id={}",
                idx + 1,
                artifacts.len(),
                artifact.artifact_id
            ),
        )
        .await
        .ok();
    }

    let artifacts_for_task = if artifacts.is_empty() {
        log.warn(
            "sse_complete",
            "Artifacts array is EMPTY, setting Task.artifacts to None",
        )
        .await
        .ok();
        None
    } else {
        log.info(
            "sse_complete",
            &format!("Setting Task.artifacts to Some({} items)", artifacts.len()),
        )
        .await
        .ok();
        Some(artifacts.clone())
    };

    let task_metadata = match TaskMetadata::new_validated_agent_message(agent_name.to_string()) {
        Ok(metadata) => metadata,
        Err(e) => {
            log.error(
                "sse_complete",
                &format!("Failed to create TaskMetadata: {e}"),
            )
            .await
            .ok();
            send_error(tx, &format!("Internal error: {e}"), request_id);
            return CompletionResult::Break;
        },
    };

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
                message_id: message_id.to_string(),
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
            original_message.clone(),
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
            },
        ]),
        artifacts: artifacts_for_task,
        metadata: Some(task_metadata),
    };

    let artifacts_count = complete_task.artifacts.as_ref().map(|a| a.len());
    log.info(
        "sse_complete",
        &format!("Task.artifacts count: {artifacts_count:?}"),
    )
    .await
    .ok();

    if let Some(ref metadata) = complete_task.metadata {
        if let Err(e) = metadata.validate() {
            log.error(
                "sse_complete",
                &format!("Task metadata validation failed: {e}"),
            )
            .await
            .ok();
            send_error(
                tx,
                &format!("Task metadata validation failed: {e}"),
                request_id,
            );
            return CompletionResult::Break;
        }
    } else {
        log.error("sse_complete", "Task metadata is None before SSE send")
            .await
            .ok();
        send_error(tx, "Task metadata cannot be None", request_id);
        return CompletionResult::Break;
    }

    let agent_message = match complete_task.status.message.clone() {
        Some(msg) => msg,
        None => {
            log.error("sse_complete", "Task status message is None")
                .await
                .ok();
            send_error(tx, "Task status message cannot be None", request_id);
            return CompletionResult::Break;
        },
    };

    match processor
        .persist_completed_task(
            &complete_task,
            original_message,
            &agent_message,
            context,
            agent_name,
            true,
        )
        .await
    {
        Err(e) => {
            log.error(
                "sse_complete",
                &format!("Failed to complete task and persist messages: {e}"),
            )
            .await
            .ok();

            let failed_timestamp = chrono::Utc::now();
            if let Err(update_err) = task_repo
                .update_task_state(task_id.as_str(), TaskState::Failed, &failed_timestamp)
                .await
            {
                log.error(
                    "sse_complete",
                    &format!("Failed to update task to failed state: {update_err}"),
                )
                .await
                .ok();
            }

            send_error(
                tx,
                &format!("Failed to persist task completion: {e}"),
                request_id,
            );
            CompletionResult::Break
        },
        Ok(task_with_timing) => {
            log.info(
                "sse_complete",
                &format!(
                    "Task {} completed and persisted with timing: execution_time_ms = {:?}",
                    task_id,
                    task_with_timing
                        .metadata
                        .as_ref()
                        .and_then(|m| m.execution_time_ms)
                ),
            )
            .await
            .ok();

            let complete_event = json!({
                "jsonrpc": "2.0",
                "result": task_with_timing,
                "id": request_id
            });

            let artifacts_info = complete_event
                .get("result")
                .and_then(|r| r.get("artifacts"))
                .map(|a| {
                    if a.is_null() {
                        "null".to_string()
                    } else if let Some(arr) = a.as_array() {
                        format!("array[{}]", arr.len())
                    } else {
                        format!("{a:?}")
                    }
                });
            log.info(
                "sse_complete",
                &format!(
                    "Sending JSON to frontend: artifacts in result = {:?}",
                    artifacts_info
                ),
            )
            .await
            .ok();

            let _ = tx.send(Event::default().data(complete_event.to_string()));

            broadcast_task_completed(&task_with_timing, context.user_id().as_str(), token, log)
                .await;

            CompletionResult::Break
        },
    }
}

pub async fn handle_error(
    tx: &UnboundedSender<Event>,
    error: String,
    task_id: &TaskId,
    context_id: &ContextId,
    request_id: &Option<serde_json::Value>,
    log: &LogService,
    task_repo: &TaskRepository,
) {
    let failed_timestamp = chrono::Utc::now();
    if let Err(e) = task_repo
        .update_task_state(task_id.as_str(), TaskState::Failed, &failed_timestamp)
        .await
    {
        log.error(
            "sse_error",
            &format!("Failed to update task to failed state: {e}"),
        )
        .await
        .ok();
    }

    log.error("sse_error", &format!("Task {task_id} failed: {error}"))
        .await
        .ok();

    // Emit a status-update event so frontend can clear streaming state
    let failed_event = serde_json::json!({
        "jsonrpc": "2.0",
        "result": {
            "kind": "status-update",
            "taskId": task_id,
            "contextId": context_id,
            "status": {
                "state": "failed",
                "message": error,
                "timestamp": failed_timestamp
            },
            "final": true
        },
        "id": request_id
    });
    let _ = tx.send(Event::default().data(failed_event.to_string()));

    // Also send JSON-RPC error for error banner display
    send_error(tx, &error, request_id);
}

fn send_error(tx: &UnboundedSender<Event>, message: &str, request_id: &Option<serde_json::Value>) {
    let error_event = json!({
        "jsonrpc": "2.0",
        "error": {
            "code": -32603,
            "message": message
        },
        "id": request_id
    });
    let _ = tx.send(Event::default().data(error_event.to_string()));
}
