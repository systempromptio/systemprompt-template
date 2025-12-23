use anyhow::Result;
use chrono::Utc;
use rmcp::{
    model::{CallToolRequestParam, CallToolResult, ListToolsResult, PaginatedRequestParam},
    service::RequestContext,
    ErrorData as McpError, RoleServer,
};
use systemprompt::database::DbPool;
use systemprompt::mcp::middleware::enforce_rbac_from_registry;
use systemprompt::mcp::models::{ToolExecutionRequest, ToolExecutionResult};
use systemprompt::mcp::repository::ToolUsageRepository;

use crate::server::InfrastructureServer;

impl InfrastructureServer {
    #[allow(clippy::unused_async)]
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

        let auth_result = enforce_rbac_from_registry(&ctx, self.service_id.as_str()).await?;
        let authenticated_ctx = auth_result.expect_authenticated(
            "BUG: systemprompt-infrastructure requires OAuth but auth was not enforced",
        )?;

        let request_context = authenticated_ctx.context.clone();

        let output_schema = self.get_output_schema_for_tool(&tool_name);

        let tool_repo = ToolUsageRepository::new(DbPool::clone(&self.db_pool)).map_err(|e| {
            McpError::internal_error(format!("Failed to create tool repository: {e}"), None)
        })?;
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

        let result = crate::tools::handle_tool_call(
            &tool_name,
            request,
            ctx,
            &self.sync_service,
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

        let status_str = if result.is_ok() { "success" } else { "error" };
        tracing::info!(
            tool_name = %tool_name,
            status = %status_str,
            execution_id = %execution_id.as_str(),
            context_id = %request_context.execution.context_id.as_str(),
            duration_ms = %start_time.elapsed().as_millis(),
            "Tool executed"
        );

        result
    }
}
