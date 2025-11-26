use anyhow::Result;
use chrono::Utc;
use rmcp::{model::*, service::RequestContext, ErrorData as McpError, RoleServer};
use systemprompt_core_agent::services::mcp::task_helper;
use systemprompt_core_logging::LogService;
use systemprompt_core_mcp::middleware::enforce_rbac_from_registry;
use systemprompt_core_mcp::repository::ToolUsageRepository;
use systemprompt_models::execution::CallSource;

use crate::server::AdminServer;

impl AdminServer {
    pub(in crate::server) async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        crate::tools::list_tools()
    }

    pub(in crate::server) async fn call_tool(
        &self,
        request: CallToolRequestParam,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let tool_name = request.name.to_string();

        let auth_result =
            enforce_rbac_from_registry(&ctx, &self.server_name, Some(&self.system_log)).await?;
        let authenticated_ctx = auth_result.expect_authenticated(
            "BUG: systemprompt-admin requires OAuth but auth was not enforced",
        );

        let mut request_context = authenticated_ctx.context.clone();
        let jwt_token = authenticated_ctx.token();
        let logger = LogService::new(self.db_pool.clone(), request_context.log_context());

        let output_schema = self.get_output_schema_for_tool(&tool_name);

        let task_id = task_helper::ensure_task_exists(
            &self.db_pool,
            &mut request_context,
            &tool_name,
            &self.server_name,
            &logger,
        )
        .await?;

        let tool_repo = ToolUsageRepository::new(self.db_pool.clone());
        let arguments = request.arguments.as_ref().cloned().unwrap_or_default();

        let exec_request = systemprompt_core_mcp::repository::ToolExecutionRequest {
            tool_name: tool_name.clone(),
            mcp_server_name: self.server_name.clone(),
            input: serde_json::Value::Object(arguments.clone()),
            started_at: Utc::now(),
            context: request_context.clone(),
            request_method: Some("call_tool".to_string()),
            request_source: Some("mcp_server".to_string()),
            ai_tool_call_id: request_context.ai_tool_call_id().cloned(),
        };

        let execution_id = tool_repo
            .start_execution(&exec_request)
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to start execution: {}", e), None)
            })?;

        let mut result =
            crate::tools::handle_tool_call(&tool_name, request, ctx, logger.clone(), &self.db_pool)
                .await;

        inject_execution_id_into_result(&mut result, &execution_id);

        let exec_result = systemprompt_core_mcp::repository::ToolExecutionResult {
            output: result
                .as_ref()
                .ok()
                .and_then(|r| r.structured_content.clone()),
            output_schema: output_schema.clone(),
            status: if result.is_ok() { "success" } else { "failed" }.to_string(),
            error_message: result.as_ref().err().map(|e| format!("{:?}", e)),
            completed_at: Utc::now(),
        };

        tool_repo
            .complete_execution(&execution_id, &exec_result)
            .await
            .ok();

        logger
            .info(
                "mcp_admin",
                &format!(
                    "✓ Tool execution completed: {}",
                    if result.is_ok() { "success" } else { "error" }
                ),
            )
            .await
            .ok();

        if let Ok(ref mut tool_result) = result {
            let call_source = request_context.call_source().unwrap_or(CallSource::Agentic);

            match call_source {
                CallSource::Ephemeral => {
                    logger
                        .info(
                            "mcp_admin",
                            "Ephemeral call - injecting artifact type from schema",
                        )
                        .await
                        .ok();
                    inject_artifact_type_from_schema(tool_result, &output_schema, &logger).await;
                }
                CallSource::Agentic | CallSource::Direct => {
                    logger
                        .info(
                            "mcp_admin",
                            &format!("Processing artifact (call_source: {:?})...", call_source),
                        )
                        .await
                        .ok();

                    let tool_arguments = serde_json::Value::Object(arguments.clone());

                    match self
                        .tool_result_handler
                        .process_tool_result(
                            &tool_name,
                            tool_result,
                            output_schema.as_ref(),
                            Some(&tool_arguments),
                            &task_id,
                            &request_context.execution.context_id,
                            &request_context,
                        )
                        .await
                    {
                        Ok(artifact) => {
                            logger
                                .info(
                                    "mcp_admin",
                                    &format!(
                                        "✅ Artifact {} transformed with fingerprint: {:?}",
                                        artifact.artifact_id, artifact.metadata.fingerprint
                                    ),
                                )
                                .await
                                .ok();

                            match self
                                .publishing_service
                                .publish_from_mcp(
                                    &artifact,
                                    &task_id,
                                    &request_context.execution.context_id,
                                    &tool_name,
                                    &tool_arguments,
                                    &request_context,
                                    call_source,
                                )
                                .await
                            {
                                Ok(()) => {
                                    logger
                                        .info(
                                            "mcp_admin",
                                            &format!(
                                                "✅ Artifact {} published (persisted + messages)",
                                                artifact.artifact_id
                                            ),
                                        )
                                        .await
                                        .ok();
                                }
                                Err(e) => {
                                    logger
                                        .warn(
                                            "mcp_admin",
                                            &format!("Failed to publish artifact: {}", e),
                                        )
                                        .await
                                        .ok();
                                }
                            }
                        }
                        Err(e) => {
                            logger
                                .warn(
                                    "mcp_admin",
                                    &format!("Failed to transform tool result to artifact: {}", e),
                                )
                                .await
                                .ok();
                        }
                    }
                }
            }
        }

        let server = self.clone();
        let task_id_clone = task_id.clone();
        let logger_clone = logger.clone();
        let jwt_token_clone = jwt_token.to_string();

        tokio::spawn(async move {
            logger_clone
                .info("mcp_admin", "Completing task in background...")
                .await
                .ok();

            if let Err(e) = task_helper::complete_task(
                &server.db_pool,
                &task_id_clone,
                &jwt_token_clone,
                &logger_clone,
            )
            .await
            {
                logger_clone
                    .error("mcp_admin", &format!("Failed to complete task: {:?}", e))
                    .await
                    .ok();
            } else {
                logger_clone.info("mcp_admin", "✓ Task complete").await.ok();
            }
        });

        result
    }
}

fn inject_execution_id_into_result(
    result: &mut Result<CallToolResult, McpError>,
    execution_id: &systemprompt_identifiers::McpExecutionId,
) {
    if let Ok(ref mut tool_result) = result {
        if let Some(ref mut content) = tool_result.structured_content {
            if let Some(obj) = content.as_object_mut() {
                obj.insert(
                    "mcp_execution_id".to_string(),
                    serde_json::json!(execution_id.to_string()),
                );
            }
        }
    }
}

async fn inject_artifact_type_from_schema(
    tool_result: &mut CallToolResult,
    output_schema: &Option<serde_json::Value>,
    logger: &LogService,
) {
    if let Some(ref schema) = output_schema {
        if let Some(artifact_type) = schema.get("x-artifact-type") {
            if let Some(ref mut structured) = tool_result.structured_content {
                if let Some(obj) = structured.as_object_mut() {
                    obj.insert("x-artifact-type".to_string(), artifact_type.clone());
                    logger
                        .info(
                            "mcp_admin",
                            &format!("Injected x-artifact-type: {:?}", artifact_type),
                        )
                        .await
                        .ok();
                }
            }
        }
    }
}
