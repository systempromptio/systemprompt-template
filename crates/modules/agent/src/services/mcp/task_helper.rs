use crate::models::a2a::{Artifact, Message, Part, Task, TaskState, TaskStatus, TextPart};
use crate::repository::{ContextRepository, TaskRepository};
use crate::services::MessageService;
use rmcp::ErrorData as McpError;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::Config;
use systemprompt_identifiers::{ContextId, SessionId, TaskId, TraceId, UserId};
use systemprompt_models::auth::UserType;
use systemprompt_models::TaskMetadata;
use uuid::Uuid;

/// Result of ensure_task_exists - includes whether task was newly created
#[derive(Debug)]
pub struct TaskResult {
    pub task_id: TaskId,
    /// True if this MCP tool created the task, false if reused from parent
    /// (e.g., A2A agent)
    pub is_owner: bool,
}

pub async fn ensure_task_exists(
    db_pool: &DbPool,
    request_context: &mut systemprompt_models::execution::context::RequestContext,
    tool_name: &str,
    mcp_server_name: &str,
    logger: &LogService,
) -> Result<TaskResult, McpError> {
    if let Some(task_id) = request_context.task_id() {
        logger
            .info(
                "mcp_task",
                &format!("Task reused from parent | task_id={}", task_id.as_str()),
            )
            .await
            .ok();
        return Ok(TaskResult {
            task_id: task_id.clone(),
            is_owner: false,
        });
    }

    if !matches!(
        request_context.user_type(),
        UserType::Unknown | UserType::Anon
    ) {
        let context_repo = ContextRepository::new(db_pool.clone());

        if let Err(_) = context_repo
            .validate_context_ownership(
                request_context.context_id().as_str(),
                request_context.user_id().as_str(),
            )
            .await
        {
            let error_msg = format!(
                "Invalid context_id '{}'. Please select a valid context for this user. Create a \
                 context via POST /api/v1/core/contexts or list existing contexts via GET \
                 /api/v1/core/contexts",
                request_context.context_id().as_str()
            );

            logger.error("mcp_task", &error_msg).await.ok();

            return Err(McpError::invalid_params(error_msg, None));
        }
    }

    let task_repo = TaskRepository::new(db_pool.clone());

    let task_id = TaskId::generate();

    let agent_name = request_context.agent_name().to_string();

    let metadata = TaskMetadata::new_mcp_execution(
        agent_name.clone(),
        tool_name.to_string(),
        mcp_server_name.to_string(),
    );

    let task = Task {
        id: task_id.clone(),
        context_id: request_context.context_id().clone(),
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

    task_repo
        .create_task(
            &task,
            &UserId::new(request_context.user_id().as_str()),
            &SessionId::new(request_context.session_id().as_str()),
            &TraceId::new(request_context.trace_id().as_str()),
            &agent_name,
        )
        .await
        .map_err(|e| McpError::internal_error(format!("Failed to create task: {e}"), None))?;

    request_context.execution.task_id = Some(task_id.clone());

    logger
        .info(
            "mcp_task",
            &format!(
                "Task created | task_id={}, tool={}, agent={}",
                task_id.as_str(),
                tool_name,
                agent_name
            ),
        )
        .await
        .ok();

    Ok(TaskResult {
        task_id,
        is_owner: true,
    })
}

pub async fn complete_task(
    db_pool: &DbPool,
    task_id: &TaskId,
    jwt_token: &str,
    logger: &LogService,
) -> Result<(), McpError> {
    if let Err(e) = trigger_task_completion_broadcast(db_pool, task_id, jwt_token, logger).await {
        logger
            .error(
                "mcp_task",
                &format!(
                    "Webhook broadcast failed | task_id={}, error={:?}",
                    task_id.as_str(),
                    e
                ),
            )
            .await
            .ok();
    }

    Ok(())
}

async fn trigger_task_completion_broadcast(
    db_pool: &DbPool,
    task_id: &TaskId,
    jwt_token: &str,
    logger: &LogService,
) -> Result<(), McpError> {
    let task_repo = TaskRepository::new(db_pool.clone());

    let task_info = task_repo
        .get_task_context_info(task_id.as_str())
        .await
        .map_err(|e| {
            McpError::internal_error(format!("Failed to load task for webhook: {e}"), None)
        })?;

    if let Some(info) = task_info {
        let context_id = info.context_id;
        let user_id = info.user_id;

        let config = Config::global();
        let webhook_url = format!("{}/api/v1/webhook/broadcast", config.api_server_url);
        let webhook_payload = serde_json::json!({
            "event_type": "task_completed",
            "entity_id": task_id.as_str(),
            "context_id": context_id,
            "user_id": user_id,
        });

        logger
            .debug(
                "mcp_task",
                &format!(
                    "Webhook triggering | task_id={}, context_id={}",
                    task_id.as_str(),
                    context_id
                ),
            )
            .await
            .ok();

        let client = reqwest::Client::new();
        match client
            .post(webhook_url)
            .header("Authorization", format!("Bearer {jwt_token}"))
            .json(&webhook_payload)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    logger
                        .debug(
                            "mcp_task",
                            &format!(
                                "Task completed | task_id={}, webhook=success",
                                task_id.as_str()
                            ),
                        )
                        .await
                        .ok();
                } else {
                    let status = response.status();
                    logger
                        .error(
                            "mcp_task",
                            &format!(
                                "Task completed | task_id={}, webhook=failed, status={}",
                                task_id.as_str(),
                                status
                            ),
                        )
                        .await
                        .ok();
                }
            },
            Err(e) => {
                logger
                    .error(
                        "mcp_task",
                        &format!("Webhook failed | task_id={}, error={}", task_id.as_str(), e),
                    )
                    .await
                    .ok();
            },
        }
    }

    Ok(())
}

pub async fn save_messages_for_tool_execution(
    db_pool: &DbPool,
    task_id: &TaskId,
    context_id: &ContextId,
    tool_name: &str,
    tool_result: &str,
    artifact: Option<&Artifact>,
    user_id: &str,
    session_id: &str,
    trace_id: &str,
    logger: &LogService,
) -> Result<(), McpError> {
    let message_service = MessageService::new(db_pool.clone(), logger.clone());

    let user_message = Message {
        role: "user".to_string(),
        parts: vec![Part::Text(TextPart {
            text: format!("Execute tool: {tool_name}"),
        })],
        message_id: Uuid::new_v4().to_string(),
        task_id: Some(task_id.clone()),
        context_id: context_id.clone(),
        kind: "message".to_string(),
        metadata: None,
        extensions: None,
        reference_task_ids: None,
    };

    let agent_text = if let Some(artifact) = artifact {
        format!(
            "Tool execution completed. Result: {}\n\nArtifact created: {} (type: {})",
            tool_result, artifact.artifact_id, artifact.metadata.artifact_type
        )
    } else {
        format!("Tool execution completed. Result: {tool_result}")
    };

    let agent_message = Message {
        role: "agent".to_string(),
        parts: vec![Part::Text(TextPart { text: agent_text })],
        message_id: Uuid::new_v4().to_string(),
        task_id: Some(task_id.clone()),
        context_id: context_id.clone(),
        kind: "message".to_string(),
        metadata: None,
        extensions: None,
        reference_task_ids: None,
    };

    message_service
        .persist_messages(
            task_id,
            context_id,
            vec![user_message, agent_message],
            Some(user_id),
            session_id,
            trace_id,
        )
        .await
        .map_err(|e| McpError::internal_error(format!("Failed to save messages: {e}"), None))?;

    Ok(())
}
