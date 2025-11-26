use axum::{
    extract::{Extension, State},
    response::sse::{Event, Sse},
    response::IntoResponse,
};
use serde_json::json;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use systemprompt_core_agent::models::context::ContextStateEvent;
use systemprompt_core_agent::repository::{ArtifactRepository, ContextRepository, TaskRepository};
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum};
use systemprompt_core_logging::LogService;
use systemprompt_core_system::{RequestContext, CONTEXT_BROADCASTER};

pub async fn stream_context_state(
    Extension(request_context): Extension<RequestContext>,
    State(app_context): State<systemprompt_core_system::AppContext>,
) -> impl IntoResponse {
    let task_repo = std::sync::Arc::new(TaskRepository::new(app_context.db_pool().clone()));
    let artifact_repo = std::sync::Arc::new(ArtifactRepository::new(app_context.db_pool().clone()));
    let repo = ContextRepository::new(app_context.db_pool().clone(), task_repo, artifact_repo);

    let user_id = request_context.user_id().as_str().to_string();
    let logger = LogService::new(app_context.db_pool().clone(), request_context.log_context());
    let conn_id = uuid::Uuid::new_v4().to_string();

    logger
        .info(
            "context_stream",
            &format!(
                "Opening SSE stream for user {} (connection: {})",
                user_id, conn_id
            ),
        )
        .await
        .ok();

    let (tx, rx) = mpsc::unbounded_channel();

    CONTEXT_BROADCASTER
        .register(&user_id, &conn_id, tx.clone())
        .await;

    // Send complete context snapshot with accurate stats immediately
    match repo.list_contexts_with_stats(&user_id).await {
        Ok(contexts_with_stats) => {
            let snapshot = json!({
                "type": "snapshot",
                "contexts": contexts_with_stats.iter().map(|c| json!({
                    "context_id": c.context_id,
                    "name": c.name,
                    "created_at": c.created_at,
                    "updated_at": c.updated_at,
                    "message_count": c.message_count,
                    "task_count": c.task_count,
                })).collect::<Vec<_>>(),
                "timestamp": chrono::Utc::now().to_rfc3339()
            });

            logger
                .info(
                    "context_stream",
                    &format!(
                        "Sent complete snapshot with {} contexts and stats (connection: {})",
                        contexts_with_stats.len(),
                        conn_id
                    ),
                )
                .await
                .ok();

            if tx
                .send(Ok(Event::default()
                    .event("snapshot")
                    .data(snapshot.to_string())))
                .is_err()
            {
                logger
                    .error(
                        "context_stream",
                        &format!("Failed to send snapshot (connection: {})", conn_id),
                    )
                    .await
                    .ok();
            }

            // Send current_agent event for each context
            for context in &contexts_with_stats {
                let query =
                    DatabaseQueryEnum::GetLastAgentForContext.get(app_context.db_pool().as_ref());
                match app_context
                    .db_pool()
                    .fetch_optional(&query, &[&context.context_id])
                    .await
                {
                    Ok(row) => {
                        let agent_name = row.and_then(|r| {
                            r.get("agent_name")
                                .and_then(|v| v.as_str().map(String::from))
                        });

                        let event = ContextStateEvent::CurrentAgent {
                            context_id: context.context_id.clone(),
                            agent_name,
                            timestamp: chrono::Utc::now(),
                        };

                        let event_json =
                            serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string());

                        if tx
                            .send(Ok(Event::default().event("current_agent").data(event_json)))
                            .is_err()
                        {
                            logger
                                .error(
                                    "context_stream",
                                    &format!("Failed to send current_agent event for context {} (connection: {})", context.context_id, conn_id),
                                )
                                .await
                                .ok();
                        }
                    },
                    Err(e) => {
                        logger
                            .error(
                                "context_stream",
                                &format!(
                                    "Failed to query last agent for context {}: {}",
                                    context.context_id, e
                                ),
                            )
                            .await
                            .ok();
                    },
                }
            }
        },
        Err(e) => {
            logger
                .error(
                    "context_stream",
                    &format!(
                        "Failed to create snapshot for connection {}: {}",
                        conn_id, e
                    ),
                )
                .await
                .ok();
        },
    }

    let stream = UnboundedReceiverStream::new(rx);

    Sse::new(stream)
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(15))
                .event(
                    Event::default()
                        .event("heartbeat")
                        .data(json!({"type": "heartbeat"}).to_string()),
                ),
        )
        .into_response()
}
