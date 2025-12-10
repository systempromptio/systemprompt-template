/// # MCP Server Template - World-Class Implementation Pattern
///
/// This template demonstrates the complete, production-ready pattern for MCP servers in SystemPrompt.
///
/// ## Architecture Overview
///
/// ### 1. Middleware Stack (executed in order)
/// - **RBAC Middleware** (`enforce_rbac_from_registry`):
///   - Validates JWT token from Authorization header
///   - Extracts user_id, session_id, context_id from JWT claims
///   - Enforces role-based access control (admin/user roles)
///   - Populates `RequestContext` with validated user data
///
/// - **Task Context Helper** (`task_helper::ensure_task_exists`):
///   - Ensures task_id exists BEFORE tool execution
///   - Creates new task if none exists (direct MCP calls)
///   - Guarantees task_id is available for artifact persistence
///
/// ### 2. Tool Execution Flow
/// ```
/// Client Request (JWT in Authorization header)
///     ↓
/// RBAC Middleware (validates JWT, extracts user)
///     ↓
/// Task Helper (ensures task exists)
///     ↓
/// Tool Discovery & Execution (your tool logic)
///     ↓
/// ToolResultHandler (creates & persists artifact)
///     ↓
/// Artifact Broadcasting (SSE stream to frontend)
///     ↓
/// Task Completion (mark task as completed)
///     ↓
/// Response to Client
/// ```
///
/// ### 3. Artifact Processing (Unified Handler)
/// - **ToolResultHandler**: Single source of truth for artifact creation
///   - Transforms MCP result → A2A Artifact (McpToA2aTransformer)
///   - Validates artifact metadata (context_id, task_id, user_id)
///   - Persists artifact immediately to database
///   - Returns artifact for streaming
///
/// - **Artifact Streaming**: Real-time SSE broadcast to frontend
///   - Uses CONTEXT_BROADCASTER singleton
///   - Sends "artifact_created" event to user's SSE stream
///   - Frontend receives artifact and renders immediately
///
/// ### 4. Security Requirements
/// - **JWT Validation**: All tool calls require valid JWT (enforced by RBAC middleware)
/// - **User Validation**: No anonymous or system users for direct tool execution
/// - **Context Ownership**: User must own the context_id (validated by RBAC)
/// - **Task Integrity**: Task must exist before tool execution (enforced by helper)
///
/// ### 5. Database Effects
/// - **tool_executions**: Start/complete execution tracking (ToolUsageRepository)
/// - **task_artifacts**: Artifact persistence (ToolResultHandler → ArtifactRepository)
/// - **agent_tasks**: Task creation and completion (TaskRepository via helper)
/// - **logs**: All operations logged with session/trace/user context
///
/// ### 6. Output Schema Pattern
/// For tools returning structured_content (artifacts):
/// - Define output_schema in Tool definition (tools/mod.rs)
/// - Map tool_name → schema in `get_output_schema_for_tool()`
/// - Schema is used by McpToA2aTransformer to infer artifact type
/// - Without schema: artifact defaults to "json" type
/// - With schema: can be "table", "chart", "form", etc.
///
/// ## Implementation Checklist
/// - [x] RBAC middleware for JWT validation
/// - [x] Task helper for task_id guarantee
/// - [x] ToolResultHandler for unified artifact processing
/// - [x] Artifact streaming to frontend via SSE
/// - [x] Tool execution tracking (start/complete)
/// - [x] Task completion for direct calls
/// - [x] Output schema mapping for artifact rendering
/// - [x] Comprehensive logging with context
///
/// ## Key Dependencies
/// - `systemprompt_core_agent::services::mcp::ToolResultHandler` - Artifact processing
/// - `systemprompt_core_mcp::middleware::enforce_rbac_from_registry` - JWT/RBAC
/// - `systemprompt_core_agent::services::mcp::task_helper` - Task management
/// - `systemprompt_core_system::services::CONTEXT_BROADCASTER` - SSE streaming

use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;
use rmcp::{model::*, service::RequestContext, ErrorData as McpError, RoleServer, ServerHandler};

use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;
use systemprompt_core_mcp::middleware::{enforce_rbac_from_registry, AuthResult};
use systemprompt_core_mcp::repository::ToolUsageRepository;
use systemprompt_core_agent::services::mcp::task_helper;

use crate::prompts::TemplatePrompts;
use crate::tools::TemplateTools;

#[derive(Clone)]
pub struct TemplateServer {
    db_pool: DbPool,
    server_name: String,
    tools: Arc<TemplateTools>,
    prompts: Arc<TemplatePrompts>,
    system_log: LogService,
    tool_result_handler: Arc<systemprompt_core_agent::services::mcp::ToolResultHandler>,
}

impl TemplateServer {
    pub fn new(db_pool: DbPool, server_name: String) -> Self {
        let prompts = Arc::new(TemplatePrompts::new(db_pool.clone(), server_name.clone()));
        let tools = Arc::new(TemplateTools::new(db_pool.clone(), prompts.clone()));
        let system_log = LogService::system(db_pool.clone());

        let tool_result_handler = Arc::new(
            systemprompt_core_agent::services::mcp::ToolResultHandler::new(
                db_pool.clone(),
                system_log.clone(),
            )
        );

        Self {
            db_pool,
            server_name,
            tools,
            prompts,
            system_log,
            tool_result_handler,
        }
    }

    /// Resolves the output_schema for a tool based on its definition and runtime arguments.
    ///
    /// CRITICAL FOR ARTIFACT RENDERING:
    /// - This method MUST return the output_schema for any tool that returns structured_content
    /// - The schema is used to create artifacts in the frontend
    /// - Without the schema, artifacts will NOT render even if the tool returns valid data
    ///
    /// Pattern:
    /// 1. Add `output_schema: Some(Arc::new(...))` to your Tool definition in tools/mod.rs
    /// 2. Create a helper function like `pub fn my_tool_output_schema() -> serde_json::Value`
    /// 3. Map the tool name to the schema in this method
    /// 4. For dynamic schemas, check arguments to determine which schema to return
    ///
    /// Example:
    /// ```rust
    /// match tool_name {
    ///     "my_artifact_tool" => Some(tools::my_tool_output_schema()),
    ///     "dynamic_tool" => arguments
    ///         .get("type")
    ///         .and_then(|v| v.as_str())
    ///         .and_then(tools::get_schema_for_type),
    ///     _ => None,
    /// }
    /// ```
    fn get_output_schema_for_tool(
        &self,
        _tool_name: &str,
        _arguments: &serde_json::Map<String, serde_json::Value>,
    ) -> Option<serde_json::Value> {
        None
    }
}

impl ServerHandler for TemplateServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: format!("Template MCP Server ({})", self.server_name),
                version: "1.0.0".to_string(),
                icons: None,
                title: None,
                website_url: None,
            },
            instructions: Some(
                "Template MCP server. Customize the server description and instructions here."
                    .to_string(),
            ),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        self.system_log
            .info("mcp_initialize", "=== INITIALIZE CALLED ===")
            .await
            .ok();

        if let Some(parts) = context.extensions.get::<axum::http::request::Parts>() {
            self.system_log
                .info(
                    "mcp_template",
                    &format!(
                        "MCP server initialized - URI: {}, server: {}",
                        parts.uri, self.server_name
                    ),
                )
                .await
                .ok();
        }

        self.system_log
            .info("mcp_initialize", "=== INITIALIZE COMPLETE ===")
            .await
            .ok();
        Ok(self.get_info())
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        self.tools.list_tools().await
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let tool_name = request.name.to_string();

        let auth_result = enforce_rbac_from_registry(&ctx, &self.server_name, Some(&self.system_log)).await?;
        let authenticated_ctx = auth_result.expect_authenticated(
            "BUG: Template MCP server requires OAuth but auth was not enforced"
        );

        let mut request_context = authenticated_ctx.context.clone();
        let jwt_token = authenticated_ctx.token();
        let logger = LogService::new(self.db_pool.clone(), request_context.log_context());

        let output_schema = request
            .arguments
            .as_ref()
            .and_then(|args| self.get_output_schema_for_tool(&tool_name, args));

        let arguments = request.arguments.as_ref().cloned().unwrap_or_default();

        let task_id = task_helper::ensure_task_exists(
            &self.db_pool,
            &mut request_context,
            &tool_name,
            &self.server_name,
            &logger,
        )
        .await?;

        let tool_repo = ToolUsageRepository::new(self.db_pool.clone());
        let exec_request = systemprompt_core_mcp::repository::ToolExecutionRequest {
            tool_name: tool_name.clone(),
            mcp_server_name: self.server_name.clone(),
            input: serde_json::Value::Object(arguments.clone()),
            started_at: Utc::now(),
            context: request_context.clone(),
            request_method: Some("call_tool".to_string()),
            request_source: Some("mcp_server".to_string()),
        };

        let execution_id = tool_repo
            .start_execution(&exec_request)
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to start execution: {}", e), None))?;

        let result = self
            .tools
            .call_tool(&tool_name, request, ctx, logger.clone())
            .await;

        let exec_result = systemprompt_core_mcp::repository::ToolExecutionResult {
            output: result.as_ref().ok().and_then(|r| r.structured_content.clone()),
            output_schema: output_schema.clone(),
            status: if result.is_ok() { "success" } else { "failed" }.to_string(),
            error_message: result.as_ref().err().map(|e| format!("{:?}", e)),
            completed_at: Utc::now(),
        };

        tool_repo.complete_execution(&execution_id, &exec_result).await.ok();

        // Process tool result and persist artifact immediately
        if let Ok(ref tool_result) = result {
            match self.tool_result_handler.process_tool_result(
                &tool_name,
                tool_result,
                output_schema.as_ref(),
                &task_id,
                &request_context.context_id,
                &request_context,
            ).await {
                Ok(artifact) => {
                    logger.info(
                        "mcp_server",
                        &format!("Artifact {} created and persisted for direct tool call", artifact.artifact_id)
                    ).await.ok();

                    // Broadcast artifact to frontend
                    use systemprompt_core_system::services::{CONTEXT_BROADCASTER, StreamEvent};

                    let artifact_event = StreamEvent {
                        event_type: "artifact_created".to_string(),
                        context_id: request_context.context_id.to_string(),
                        user_id: request_context.user_id.to_string(),
                        data: serde_json::to_value(&artifact).unwrap_or_default(),
                        timestamp: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    };

                    CONTEXT_BROADCASTER
                        .broadcast_to_user(request_context.user_id.as_str(), artifact_event)
                        .await;
                }
                Err(e) => {
                    logger.warn(
                        "mcp_server",
                        &format!("Failed to process tool result: {}", e)
                    ).await.ok();
                }
            }

            // Direct call: Complete task immediately
            task_helper::complete_task(&self.db_pool, &task_id, jwt_token, &logger).await.ok();
        }

        result
    }

    async fn list_prompts(
        &self,
        request: Option<PaginatedRequestParam>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        self.prompts.list_prompts(request, ctx).await
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        self.prompts.get_prompt(request, ctx).await
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            next_cursor: None,
            resources: Vec::new(),
        })
    }

    async fn read_resource(
        &self,
        _request: ReadResourceRequestParam,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        Err(McpError::invalid_params(
            "Resources not supported by this server".to_string(),
            None,
        ))
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParam>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult {
            next_cursor: None,
            resource_templates: Vec::new(),
        })
    }
}
