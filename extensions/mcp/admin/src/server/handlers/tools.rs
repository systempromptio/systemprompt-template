use anyhow::Result;
use chrono::Utc;
use rmcp::{model::{PaginatedRequestParam, ListToolsResult, CallToolRequestParam, CallToolResult}, service::RequestContext, ErrorData as McpError, RoleServer};
use systemprompt_core_agent::services::mcp::task_helper;
use systemprompt_core_mcp::middleware::enforce_rbac_from_registry;
use systemprompt_core_mcp::models::{ToolExecutionRequest, ToolExecutionResult};
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
        let start_time = std::time::Instant::now();
        let tool_name = request.name.to_string();

        let auth_result =
            enforce_rbac_from_registry(&ctx, self.service_id.as_str(), None)
                .await?;
        let authenticated_ctx = auth_result.expect_authenticated(
            "BUG: systemprompt-admin requires OAuth but auth was not enforced",
        );

        let mut request_context = authenticated_ctx.context.clone();
        let jwt_token = authenticated_ctx.token();

        let output_schema = self.get_output_schema_for_tool(&tool_name);

        let task_result = task_helper::ensure_task_exists(
            &self.db_pool,
            &mut request_context,
            &tool_name,
            self.service_id.as_str(),
        )
        .await?;
        let task_id = task_result.task_id.clone();
        let is_task_owner = task_result.is_owner;

        let tool_repo = ToolUsageRepository::new(self.db_pool.clone());
        let arguments = request.arguments.clone().unwrap_or_default();

        let exec_request = ToolExecutionRequest {
            tool_name: tool_name.clone(),
            mcp_server_name: self.service_id.to_string(),
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
                McpError::internal_error(format!("Failed to start execution: {e}"), None)
            })?;

        let mut result = crate::tools::handle_tool_call(
            &tool_name,
            request,
            ctx,
            &self.db_pool,
            &self.app_context,
            &execution_id,
        )
        .await;

        let exec_result = ToolExecutionResult {
            output: result
                .as_ref()
                .ok()
                .and_then(|r| r.structured_content.clone()),
            output_schema: output_schema.clone(),
            status: if result.is_ok() { "success" } else { "failed" }.to_string(),
            error_message: result.as_ref().err().map(|e| format!("{e:?}")),
            completed_at: Utc::now(),
        };

        tool_repo
            .complete_execution(&execution_id, &exec_result)
            .await
            .ok();

        if let Ok(ref mut tool_result) = result {
            let call_source = request_context.call_source().unwrap_or(CallSource::Agentic);

            match call_source {
                CallSource::Ephemeral => {
                    inject_artifact_type_from_schema(tool_result, &output_schema);
                }
                CallSource::Agentic | CallSource::Direct => {
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
                                    tracing::info!(
                                        artifact_id = %artifact.artifact_id,
                                        artifact_type = %artifact.metadata.artifact_type,
                                        task_id = %task_id.as_str(),
                                        context_id = %request_context.execution.context_id.as_str(),
                                        "Artifact published"
                                    );
                                }
                                Err(e) => {
                                    tracing::warn!(error = %e, "Failed to publish artifact");
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!(error = %e, "Failed to transform tool result to artifact");
                        }
                    }
                }
            }
        }

        if is_task_owner {
            let server = self.clone();
            let task_id_clone = task_id.clone();
            let jwt_token_clone = jwt_token.to_string();

            tokio::spawn(async move {
                if let Err(e) = task_helper::complete_task(
                    &server.db_pool,
                    &task_id_clone,
                    &jwt_token_clone,
                )
                .await
                {
                    tracing::error!(error = ?e, "Failed to complete task");
                }
            });
        }

        let status_str = if result.is_ok() { "success" } else { "error" };
        tracing::info!(
            tool_name = %tool_name,
            status = %status_str,
            task_id = %task_id.as_str(),
            context_id = %request_context.execution.context_id.as_str(),
            duration_ms = %start_time.elapsed().as_millis(),
            "Tool executed"
        );

        result
    }
}

fn inject_artifact_type_from_schema(
    tool_result: &mut CallToolResult,
    output_schema: &Option<serde_json::Value>,
) {
    if let Some(ref schema) = output_schema {
        if let Some(artifact_type) = schema.get("x-artifact-type") {
            if let Some(ref mut structured) = tool_result.structured_content {
                if let Some(obj) = structured.as_object_mut() {
                    obj.insert("x-artifact-type".to_string(), artifact_type.clone());
                }
            }
        }
    }
}
