use std::sync::Arc;

use axum::response::sse::Event;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::{create_webhook_broadcaster, RequestContext};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;

use crate::models::a2a::protocol::PushNotificationConfig;
use crate::models::a2a::Message;
use crate::services::a2a_server::handlers::AgentHandlerState;
use crate::services::a2a_server::processing::message::MessageProcessor;

use super::agent_loader::load_agent_runtime;
use super::event_loop::{handle_stream_creation_error, process_events};
use super::initialization::{
    broadcast_task_created, detect_mcp_server_and_update_context, emit_message_received_event,
    emit_start_event, persist_initial_task, resolve_task_id, save_push_notification_config,
    validate_context,
};

pub async fn create_sse_stream(
    message: Message,
    agent_name: String,
    state: Arc<AgentHandlerState>,
    request_id: Option<serde_json::Value>,
    context: RequestContext,
    callback_config: Option<PushNotificationConfig>,
) -> UnboundedReceiverStream<Event> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let log = LogService::new(state.db_pool.clone(), context.log_context());

    log.info(
        "sse_stream",
        "create_sse_stream() called - spawning tokio task",
    )
    .await
    .ok();

    tokio::spawn(async move {
        let tx = tx;
        log.info("sse_stream", "Inside tokio::spawn - task execution started")
            .await
            .ok();

        let mut context = context;
        detect_mcp_server_and_update_context(&agent_name, &mut context, &log).await;

        let task_id = resolve_task_id(&message);
        let context_id = message.context_id.clone();
        let message_id = Uuid::new_v4().to_string();

        log.info(
            "sse_stream",
            &format!(
                "Generated IDs: task_id={}, context_id={}, message_id={}",
                task_id, context_id, message_id
            ),
        )
        .await
        .ok();

        if validate_context(
            &context_id,
            context.user_id().as_str(),
            &state,
            &tx,
            &request_id,
            &log,
        )
        .await
        .is_err()
        {
            drop(tx);
            return;
        }

        let task_repo = match persist_initial_task(
            &task_id,
            &context_id,
            &agent_name,
            &context,
            &state,
            &tx,
            &request_id,
            &log,
        )
        .await
        {
            Ok(repo) => repo,
            Err(()) => {
                drop(tx);
                return;
            },
        };

        broadcast_task_created(
            &task_id,
            &context_id,
            context.user_id().as_str(),
            &message,
            &agent_name,
            context.auth.auth_token.as_str(),
            &log,
        )
        .await;

        save_push_notification_config(&task_id, &callback_config, &state, &log).await;
        emit_message_received_event(&tx, &task_id, &context_id, &request_id);
        emit_start_event(&tx, &task_id, &context_id, &request_id);

        let agent_runtime =
            match load_agent_runtime(&agent_name, &task_id, &task_repo, &tx, &request_id, &log)
                .await
            {
                Ok(runtime) => runtime,
                Err(()) => {
                    drop(tx);
                    return;
                },
            };

        let broadcaster = create_webhook_broadcaster(context.auth.auth_token.as_str());
        let processor = Arc::new(MessageProcessor::new(
            state.db_pool.clone(),
            state.ai_service.clone(),
            log.clone(),
            broadcaster,
        ));

        log.info(
            "sse_stream",
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
            Ok(chunk_rx) => {
                process_events(
                    tx, chunk_rx, task_id, context_id, message_id, request_id, message, agent_name,
                    context, log, task_repo, state, processor,
                )
                .await;
            },
            Err(e) => {
                handle_stream_creation_error(
                    tx,
                    e,
                    &task_id,
                    &context_id,
                    &request_id,
                    &task_repo,
                    &log,
                )
                .await;
            },
        }
    });

    UnboundedReceiverStream::new(rx)
}
