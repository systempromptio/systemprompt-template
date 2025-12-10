use std::sync::Arc;

use axum::response::sse::Event;
use serde_json::json;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::RequestContext;
use systemprompt_identifiers::{ContextId, TaskId};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::models::a2a::{Message, TaskState};
use crate::repository::TaskRepository;
use crate::services::a2a_server::handlers::AgentHandlerState;
use crate::services::a2a_server::processing::message::{MessageProcessor, StreamEvent};

use super::handlers::{
    handle_artifact_update, handle_complete, handle_error, handle_text, handle_tool_call,
    handle_tool_result,
};

pub async fn emit_working_state(
    tx: &UnboundedSender<Event>,
    task_id: &TaskId,
    context_id: &ContextId,
    request_id: &Option<serde_json::Value>,
    task_repo: &TaskRepository,
    log: &LogService,
) {
    let working_timestamp = chrono::Utc::now();
    if let Err(e) = task_repo
        .update_task_state(task_id.as_str(), TaskState::Working, &working_timestamp)
        .await
    {
        log.error(
            "sse_loop",
            &format!("Failed to update task to working state: {e}"),
        )
        .await
        .ok();
    } else {
        log.info(
            "sse_loop",
            &format!("Task {} updated to working state", task_id),
        )
        .await
        .ok();

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
}

fn emit_response_generation_started(
    tx: &UnboundedSender<Event>,
    task_id: &TaskId,
    context_id: &ContextId,
    request_id: &Option<serde_json::Value>,
) {
    let event = json!({
        "jsonrpc": "2.0",
        "result": {
            "kind": "response_generation_started",
            "taskId": task_id,
            "contextId": context_id,
            "timestamp": chrono::Utc::now().to_rfc3339()
        },
        "id": request_id
    });
    let _ = tx.send(Event::default().data(event.to_string()));
}

fn emit_execution_step(
    tx: &UnboundedSender<Event>,
    step: crate::models::ExecutionStep,
    task_id: &TaskId,
    context_id: &ContextId,
    request_id: &Option<serde_json::Value>,
) {
    let step_event = json!({
        "jsonrpc": "2.0",
        "result": {
            "kind": "execution_step",
            "taskId": task_id,
            "contextId": context_id,
            "step": step
        },
        "id": request_id
    });
    let _ = tx.send(Event::default().data(step_event.to_string()));
}

#[allow(clippy::too_many_arguments)]
pub async fn process_events(
    tx: UnboundedSender<Event>,
    mut chunk_rx: UnboundedReceiver<StreamEvent>,
    task_id: TaskId,
    context_id: ContextId,
    message_id: String,
    request_id: Option<serde_json::Value>,
    original_message: Message,
    agent_name: String,
    context: RequestContext,
    log: LogService,
    task_repo: TaskRepository,
    state: Arc<AgentHandlerState>,
    processor: Arc<MessageProcessor>,
) {
    emit_working_state(&tx, &task_id, &context_id, &request_id, &task_repo, &log).await;
    emit_response_generation_started(&tx, &task_id, &context_id, &request_id);

    log.info("sse_loop", "Stream channel received, waiting for events...")
        .await
        .ok();

    while let Some(event) = chunk_rx.recv().await {
        match event {
            StreamEvent::Text(text) => {
                handle_text(&tx, text, &task_id, &context_id, &message_id, &request_id);
            },
            StreamEvent::ToolCallStarted(tool_call) => {
                handle_tool_call(&tx, tool_call, &request_id);
            },
            StreamEvent::ToolResult { call_id, result } => {
                handle_tool_result(&tx, call_id, result, &request_id, &log).await;
            },
            StreamEvent::ExecutionStepUpdate { step } => {
                emit_execution_step(&tx, step.clone(), &task_id, &context_id, &request_id);

                // Also broadcast to context SSE stream
                let user_id = context.user_id().as_str().to_string();
                let token = context.auth.auth_token.as_str().to_string();
                let step_clone = step;
                let task_id_clone = task_id.clone();
                let context_id_clone = context_id.clone();
                let log_clone = log.clone();
                tokio::spawn(async move {
                    super::initialization::broadcast_execution_step(
                        &step_clone,
                        &task_id_clone,
                        &context_id_clone,
                        &user_id,
                        &token,
                        &log_clone,
                    )
                    .await;
                });
            },
            StreamEvent::ArtifactUpdate {
                artifact,
                append,
                last_chunk,
            } => {
                use super::handlers::ArtifactHandleResult;
                let result = handle_artifact_update(
                    &tx,
                    artifact,
                    append,
                    last_chunk,
                    &task_id,
                    &context_id,
                    &context.user_id(),
                    &request_id,
                    &log,
                    &state,
                )
                .await;
                if matches!(result, ArtifactHandleResult::Break) {
                    break;
                }
            },
            StreamEvent::Complete {
                full_text,
                artifacts,
            } => {
                use super::handlers::CompletionResult;
                let result = handle_complete(
                    &tx,
                    full_text,
                    artifacts,
                    &task_id,
                    &context_id,
                    &message_id,
                    &request_id,
                    &original_message,
                    &agent_name,
                    &context,
                    context.auth_token().as_str(),
                    &log,
                    &task_repo,
                    &processor,
                )
                .await;
                if matches!(result, CompletionResult::Break) {
                    break;
                }
            },
            StreamEvent::Error(error) => {
                handle_error(
                    &tx,
                    error,
                    &task_id,
                    &context_id,
                    &request_id,
                    &log,
                    &task_repo,
                )
                .await;
                break;
            },
        }
    }

    drop(tx);

    log.info("sse_loop", "Stream event loop ended - all events processed")
        .await
        .ok();
}

pub async fn handle_stream_creation_error(
    tx: UnboundedSender<Event>,
    error: anyhow::Error,
    task_id: &TaskId,
    context_id: &ContextId,
    request_id: &Option<serde_json::Value>,
    task_repo: &TaskRepository,
    log: &LogService,
) {
    log.error(
        "sse_loop",
        &format!("Failed to create message stream: {error}"),
    )
    .await
    .ok();

    let failed_timestamp = chrono::Utc::now();
    if let Err(update_err) = task_repo
        .update_task_state(task_id.as_str(), TaskState::Failed, &failed_timestamp)
        .await
    {
        log.error(
            "sse_loop",
            &format!("Failed to update task to failed state: {update_err}"),
        )
        .await
        .ok();
    }

    let error_message = format!("Failed to process message: {error}");

    // Emit a status-update event so frontend can clear streaming state
    let failed_event = json!({
        "jsonrpc": "2.0",
        "result": {
            "kind": "status-update",
            "taskId": task_id,
            "contextId": context_id,
            "status": {
                "state": "failed",
                "message": error_message,
                "timestamp": failed_timestamp
            },
            "final": true
        },
        "id": request_id
    });
    let _ = tx.send(Event::default().data(failed_event.to_string()));

    // Also send JSON-RPC error for error banner display
    let error_event = json!({
        "jsonrpc": "2.0",
        "error": {
            "code": -32603,
            "message": error_message
        },
        "id": request_id
    });
    let _ = tx.send(Event::default().data(error_event.to_string()));
}
