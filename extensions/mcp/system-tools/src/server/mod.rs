pub mod constructor;

pub use constructor::SystemToolsServer;

use anyhow::Result;
use chrono::Utc;
use rmcp::{
    model::{
        CallToolRequestParam, CallToolResult, GetPromptRequestParam, GetPromptResult,
        Implementation, InitializeRequestParam, InitializeResult, ListPromptsResult,
        ListResourceTemplatesResult, ListResourcesResult, ListToolsResult, PaginatedRequestParam,
        ProtocolVersion, ReadResourceRequestParam, ReadResourceResult, ServerCapabilities,
        ServerInfo,
    },
    service::RequestContext,
    ErrorData as McpError, RoleServer, ServerHandler,
};
use systemprompt::identifiers::{AgentName, ContextId, SessionId, TraceId};
use systemprompt::mcp::middleware::enforce_rbac_from_registry;
use systemprompt::mcp::models::{ToolExecutionRequest, ToolExecutionResult};
use systemprompt::mcp::repository::ToolUsageRepository;
use systemprompt::models::execution::context::RequestContext as SysRequestContext;

impl ServerHandler for SystemToolsServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: format!("SystemPrompt System Tools MCP Server ({})", self.service_id),
                version: "1.0.0".to_string(),
                icons: None,
                title: Some("System Tools".into()),
                website_url: Some("https://systemprompt.io".to_string()),
            },
            instructions: Some(
                "File system tools for reading, writing, editing files, and searching with glob/grep patterns."
                    .to_string(),
            ),
        }
    }

    async fn initialize(
        &self,
        request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        self.system_log
            .info("mcp_initialize", "=== SYSTEM TOOLS SERVER INITIALIZE ===")
            .await
            .ok();

        if let Some(parts) = context.extensions.get::<axum::http::request::Parts>() {
            self.system_log
                .info(
                    "mcp_system_tools",
                    &format!(
                        "System Tools MCP initialized - URI: {}, server: {}",
                        parts.uri, self.service_id
                    ),
                )
                .await
                .ok();
        }

        self.system_log
            .info(
                "mcp_system_tools",
                &format!("Client: {:?}", request.client_info.name),
            )
            .await
            .ok();

        Ok(self.get_info())
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult {
            tools: crate::tools::register_tools(),
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let tool_name = request.name.to_string();

        // RBAC enforcement MUST be first operation
        let auth_result = enforce_rbac_from_registry(&context, self.service_id.as_str()).await?;
        let authenticated_ctx = auth_result.expect_authenticated(
            "system-tools requires authenticated access",
        )?;
        let request_context = authenticated_ctx.context.clone();

        let logger = self.system_log.clone();
        let arguments = request.arguments.clone().unwrap_or_default();

        // Create request context for execution tracking with tool model config
        let tool_model_config = self.get_default_model_config().map_err(|e| {
            McpError::internal_error(format!("Failed to get model config: {e}"), None)
        })?;

        let sys_request_context = request_context.with_tool_model_config(tool_model_config);

        // Start execution tracking
        let tool_repo = ToolUsageRepository::new(self.db_pool.clone());
        let exec_request = ToolExecutionRequest {
            tool_name: tool_name.clone(),
            mcp_server_name: self.service_id.to_string(),
            input: serde_json::Value::Object(arguments),
            started_at: Utc::now(),
            context: sys_request_context.clone(),
            request_method: Some("call_tool".to_string()),
            request_source: Some("mcp_server".to_string()),
            ai_tool_call_id: None,
        };

        let execution_id = tool_repo
            .start_execution(&exec_request)
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to start execution: {e}"), None)
            })?;

        // Execute tool
        let result = crate::tools::handle_tool_call(
            &tool_name,
            request,
            context,
            logger,
            self,
            &execution_id,
            sys_request_context,
        )
        .await;

        // Complete execution tracking
        let exec_result = ToolExecutionResult {
            output: result
                .as_ref()
                .ok()
                .and_then(|r| r.structured_content.clone()),
            output_schema: None,
            status: if result.is_ok() { "success" } else { "failed" }.to_string(),
            error_message: result.as_ref().err().map(|e| format!("{e:?}")),
            completed_at: Utc::now(),
        };

        tool_repo
            .complete_execution(&execution_id, &exec_result)
            .await
            .ok();

        result
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult {
            prompts: Vec::new(),
            next_cursor: None,
        })
    }

    async fn get_prompt(
        &self,
        _request: GetPromptRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        Err(McpError::invalid_params("No prompts available", None))
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: Vec::new(),
            next_cursor: None,
        })
    }

    async fn read_resource(
        &self,
        _request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        Err(McpError::invalid_params("No resources available", None))
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult {
            next_cursor: None,
            resource_templates: Vec::new(),
        })
    }
}
