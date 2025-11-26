use crate::models::a2a::{Artifact, Message, Part, Task, TaskState, TaskStatus, TextPart};
use crate::repository::{ArtifactRepository, ContextRepository, TaskRepository};
use crate::services::MessageService;
use rmcp::ErrorData as McpError;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::Config;
use systemprompt_identifiers::{ContextId, SessionId, TaskId, TraceId, UserId};
use systemprompt_models::auth::UserType;
use systemprompt_models::TaskMetadata;
use uuid::Uuid;

pub async fn ensure_task_exists(
    db_pool: &DbPool,
    request_context: &mut systemprompt_models::execution::context::RequestContext,
    tool_name: &str,
    mcp_server_name: &str,
    logger: &LogService,
) -> Result<TaskId, McpError> {
    if let Some(task_id) = request_context.task_id() {
        logger
            .info(
                "mcp_task",
                &format!("Reusing existing task from header: {}", task_id.as_str()),
            )
            .await
            .ok();
        return Ok(task_id.clone());
    }

    if !matches!(
        request_context.user_type(),
        UserType::Unknown | UserType::Anon
    ) {
        let task_repo = std::sync::Arc::new(TaskRepository::new(db_pool.clone()));
        let artifact_repo = std::sync::Arc::new(ArtifactRepository::new(db_pool.clone()));
        let context_repo = ContextRepository::new(db_pool.clone(), task_repo, artifact_repo);

        if let Err(_) = context_repo
            .validate_context_ownership(
                request_context.context_id().as_str(),
                request_context.user_id().as_str(),
            )
            .await
        {
            let error_msg = format!(
                "Invalid context_id '{}'. Please select a valid context for this user. Create a context via POST /api/v1/core/contexts or list existing contexts via GET /api/v1/core/contexts",
                request_context.context_id().as_str()
            );

            logger.error("mcp_task", &error_msg).await.ok();

            return Err(McpError::invalid_params(error_msg, None));
        }
    } else {
        logger
            .info(
                "mcp_task",
                &format!(
                    "Skipping context validation for anonymous/unknown user (user_type: {})",
                    request_context.user_type()
                ),
            )
            .await
            .ok();
    }

    let task_repo = TaskRepository::new(db_pool.clone());

    let task_id = TaskId::generate();

    // Agent name is guaranteed to be present (non-optional field in RequestContext)
    let agent_name = request_context.agent_name().as_str();

    logger
        .info(
            "mcp_task",
            &format!(
                "Creating task for MCP tool execution: agent={}, server={}, tool={}",
                agent_name, mcp_server_name, tool_name
            ),
        )
        .await
        .ok();

    let metadata = TaskMetadata::new_mcp_execution(
        agent_name.to_string(),
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
        .map_err(|e| McpError::internal_error(format!("Failed to create task: {}", e), None))?;

    logger
        .info(
            "mcp_task",
            &format!("Task created: {} (tool: {})", task_id.as_str(), tool_name),
        )
        .await
        .ok();

    Ok(task_id)
}

pub async fn complete_task(
    db_pool: &DbPool,
    task_id: &TaskId,
    jwt_token: &str,
    logger: &LogService,
) -> Result<(), McpError> {
    logger
        .info(
            "mcp_task",
            &format!(
                "Triggering webhook for task completion: {}",
                task_id.as_str()
            ),
        )
        .await
        .ok();

    if let Err(e) = trigger_task_completion_broadcast(db_pool, task_id, jwt_token, logger).await {
        eprintln!("[MCP-TASK] Failed to trigger webhook broadcast: {:?}", e);
        logger
            .error(
                "mcp_task",
                &format!("Failed to trigger webhook broadcast: {:?}", e),
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
    use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum};

    let query = DatabaseQueryEnum::GetTaskContextUser.get(db_pool.as_ref());
    let row = db_pool
        .as_ref()
        .fetch_optional(&query, &[&task_id.as_str()])
        .await
        .map_err(|e| {
            McpError::internal_error(format!("Failed to load task for webhook: {}", e), None)
        })?;

    if let Some(row) = row {
        let context_id = row
            .get("context_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::internal_error("Missing context_id".to_string(), None))?
            .to_string();

        let user_id = row
            .get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::internal_error("Missing user_id".to_string(), None))?
            .to_string();

        let config = Config::global();
        let webhook_url = format!("{}/api/v1/webhook/broadcast", config.api_server_url);
        let webhook_payload = serde_json::json!({
            "event_type": "task_completed",
            "entity_id": task_id.as_str(),
            "context_id": context_id,
            "user_id": user_id,
        });

        eprintln!(
            "[MCP-WEBHOOK] Triggering webhook for task {}",
            task_id.as_str()
        );
        eprintln!("[MCP-WEBHOOK] URL: {}", webhook_url);
        eprintln!("[MCP-WEBHOOK] JWT Token present: yes");
        eprintln!(
            "[MCP-WEBHOOK] Payload: {}",
            serde_json::to_string_pretty(&webhook_payload).unwrap_or_default()
        );

        logger
            .info(
                "mcp_task",
                &format!(
                    "🔔 Triggering webhook for task completion: {}",
                    task_id.as_str()
                ),
            )
            .await
            .ok();

        let client = reqwest::Client::new();
        match client
            .post(webhook_url)
            .header("Authorization", format!("Bearer {}", jwt_token))
            .json(&webhook_payload)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    eprintln!(
                        "[MCP-WEBHOOK] SUCCESS: Webhook returned status {}",
                        response.status()
                    );
                    logger
                        .info(
                            "mcp_task",
                            &format!(
                                "✅ Webhook triggered successfully for task {}",
                                task_id.as_str()
                            ),
                        )
                        .await
                        .ok();
                } else {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    eprintln!(
                        "[MCP-WEBHOOK] ERROR: Webhook returned status {} - {}",
                        status, body
                    );
                    logger
                        .error(
                            "mcp_task",
                            &format!(
                                "❌ Webhook returned error status: {} for task {}",
                                status,
                                task_id.as_str()
                            ),
                        )
                        .await
                        .ok();
                }
            },
            Err(e) => {
                eprintln!(
                    "[MCP-WEBHOOK] CRITICAL ERROR: Failed to trigger webhook: {}",
                    e
                );
                logger
                    .error(
                        "mcp_task",
                        &format!("❌ CRITICAL: Failed to trigger webhook: {}", e),
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

    logger
        .info(
            "mcp_task",
            &format!("Creating messages for MCP tool execution: {}", tool_name),
        )
        .await
        .ok();

    let user_message = Message {
        role: "user".to_string(),
        parts: vec![Part::Text(TextPart {
            text: format!("Execute tool: {}", tool_name),
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
        format!("Tool execution completed. Result: {}", tool_result)
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
        .map_err(|e| McpError::internal_error(format!("Failed to save messages: {}", e), None))?;

    logger
        .info(
            "mcp_task",
            &format!("Messages saved for task {}", task_id.as_str()),
        )
        .await
        .ok();

    Ok(())
}
