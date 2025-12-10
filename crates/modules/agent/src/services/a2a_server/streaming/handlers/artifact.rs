use std::sync::Arc;

use axum::response::sse::Event;
use serde_json::json;
use systemprompt_core_logging::LogService;
use systemprompt_identifiers::{ContextId, TaskId, UserId};
use systemprompt_traits::validation::Validate;
use tokio::sync::mpsc::UnboundedSender;

use crate::models::a2a::Artifact;
use crate::services::a2a_server::handlers::AgentHandlerState;
use crate::services::ArtifactPublishingService;

pub enum ArtifactHandleResult {
    Continue,
    Break,
}

pub async fn handle_artifact_update(
    tx: &UnboundedSender<Event>,
    artifact: Artifact,
    append: bool,
    last_chunk: bool,
    task_id: &TaskId,
    context_id: &ContextId,
    user_id: &UserId,
    request_id: &Option<serde_json::Value>,
    log: &LogService,
    state: &Arc<AgentHandlerState>,
) -> ArtifactHandleResult {
    let artifact_type = &artifact.metadata.artifact_type;

    if let Err(e) = artifact.metadata.validate() {
        log.error(
            "sse_artifact",
            &format!("Artifact metadata validation failed: {e}"),
        )
        .await
        .ok();

        let error_event = json!({
            "jsonrpc": "2.0",
            "error": {
                "code": -32603,
                "message": format!("Artifact metadata validation failed: {e}")
            },
            "id": request_id
        });
        let _ = tx.send(Event::default().data(error_event.to_string()));
        return ArtifactHandleResult::Break;
    }

    log.info(
        "sse_artifact",
        &format!(
            "Publishing artifact ID {} to SSE stream (type={}, parts={}, append={}, last={})",
            artifact.artifact_id,
            artifact_type,
            artifact.parts.len(),
            append,
            last_chunk
        ),
    )
    .await
    .ok();

    let publishing_service = ArtifactPublishingService::new(state.db_pool.clone(), log.clone());

    if let Err(e) = publishing_service
        .publish_from_a2a(&artifact, task_id, context_id, user_id)
        .await
    {
        log.error(
            "sse_artifact",
            &format!("Failed to persist artifact during streaming: {e}"),
        )
        .await
        .ok();

        let error_event = json!({
            "jsonrpc": "2.0",
            "error": {
                "code": -32603,
                "message": format!("Query execution failed: {e}")
            },
            "id": request_id
        });
        let _ = tx.send(Event::default().data(error_event.to_string()));
        return ArtifactHandleResult::Break;
    }

    log.info(
        "sse_artifact",
        &format!(
            "Artifact {} persisted to database before SSE emit",
            artifact.artifact_id
        ),
    )
    .await
    .ok();

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
    ArtifactHandleResult::Continue
}
