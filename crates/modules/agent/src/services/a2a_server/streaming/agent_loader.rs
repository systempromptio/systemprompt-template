use axum::response::sse::Event;
use serde_json::json;
use systemprompt_core_logging::LogService;
use systemprompt_identifiers::TaskId;
use tokio::sync::mpsc::UnboundedSender;

use crate::models::a2a::TaskState;
use crate::models::AgentRuntimeInfo;
use crate::repository::TaskRepository;
use crate::services::registry::AgentRegistry;

pub async fn load_agent_runtime(
    agent_name: &str,
    task_id: &TaskId,
    task_repo: &TaskRepository,
    tx: &UnboundedSender<Event>,
    request_id: &Option<serde_json::Value>,
    log: &LogService,
) -> Result<AgentRuntimeInfo, ()> {
    match AgentRegistry::new().await {
        Ok(registry) => match registry.get_agent(agent_name).await {
            Ok(agent_config) => Ok(agent_config.into()),
            Err(e) => {
                log.error(
                    "sse_loader",
                    &format!("Failed to load agent: {agent_name} - {e}"),
                )
                .await
                .ok();

                mark_task_failed(task_id, task_repo, log).await;

                let error_event = json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32603,
                        "message": "Agent not found"
                    },
                    "id": request_id
                });
                let _ = tx.send(Event::default().data(error_event.to_string()));
                Err(())
            },
        },
        Err(e) => {
            let error_details = format!(
                "Failed to load agent registry: {} - check if config files exist and services are \
                 properly configured",
                e
            );
            log.error("sse_loader", &error_details).await.ok();

            mark_task_failed(task_id, task_repo, log).await;

            let error_event = json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32603,
                    "message": "Failed to load agent registry - check system logs for details"
                },
                "id": request_id
            });
            let _ = tx.send(Event::default().data(error_event.to_string()));
            Err(())
        },
    }
}

pub async fn mark_task_failed(task_id: &TaskId, task_repo: &TaskRepository, log: &LogService) {
    let failed_timestamp = chrono::Utc::now();
    if let Err(update_err) = task_repo
        .update_task_state(task_id.as_str(), TaskState::Failed, &failed_timestamp)
        .await
    {
        log.error(
            "sse_loader",
            &format!("Failed to update task to failed state: {update_err}"),
        )
        .await
        .ok();
    }
}
