use axum::extract::{Extension, State};
use axum::response::sse::{Event, Sse};
use axum::response::IntoResponse;
use serde_json::json;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use systemprompt_core_agent::models::context::ContextStateEvent;
use systemprompt_core_agent::repository::ContextRepository;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::{RequestContext, CONTEXT_BROADCASTER};

fn should_sample(id: &str, sample_rate: f32) -> bool {
    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);
    let hash = hasher.finish();
    (hash as f32 / u64::MAX as f32) < sample_rate
}

pub async fn stream_context_state(
    Extension(request_context): Extension<RequestContext>,
    State(app_context): State<systemprompt_core_system::AppContext>,
) -> impl IntoResponse {
    let repo = ContextRepository::new(app_context.db_pool().clone());

    let user_id = request_context.user_id().as_str().to_string();
    let logger = LogService::new(app_context.db_pool().clone(), request_context.log_context());
    let conn_id = uuid::Uuid::new_v4().to_string();

    let should_log = should_sample(&conn_id, 0.01);
    if should_log {
        logger
            .debug(
                "context_stream",
                &format!(
                    "SSE stream opened | user_id={}, conn_id={}",
                    user_id, conn_id
                ),
            )
            .await
            .ok();
    }

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

            if should_log {
                logger
                    .debug(
                        "context_stream",
                        &format!(
                            "SSE snapshot sent | context_count={}, conn_id={}",
                            contexts_with_stats.len(),
                            conn_id
                        ),
                    )
                    .await
                    .ok();
            }

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
            let pool = app_context
                .db_pool()
                .pool_arc()
                .expect("Database must be PostgreSQL");
            for context in &contexts_with_stats {
                let agent_name_result = sqlx::query_scalar::<_, Option<String>>(
                    r"
                    SELECT agent_name
                    FROM agent_tasks
                    WHERE context_id = $1 AND agent_name IS NOT NULL
                    ORDER BY created_at DESC
                    LIMIT 1
                    ",
                )
                .bind(&context.context_id)
                .fetch_optional(&*pool)
                .await;

                match agent_name_result {
                    Ok(agent_name) => {
                        let event = ContextStateEvent::CurrentAgent {
                            context_id: context.context_id.clone(),
                            agent_name: agent_name.flatten(),
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
                                    &format!(
                                        "Failed to send current_agent event for context {} \
                                         (connection: {})",
                                        context.context_id, conn_id
                                    ),
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
