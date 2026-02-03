use crate::tools::{self, ForgetInput, GetContextInput, SearchInput, StoreMemoryInput};
use anyhow::Result;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Content, Implementation, InitializeRequestParams,
    InitializeResult, ListToolsResult, PaginatedRequestParams, ProtocolVersion, ServerCapabilities,
    ServerInfo,
};
use rmcp::service::{RequestContext as RmcpRequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler};
use systemprompt::database::DbPool;
use systemprompt::identifiers::{ArtifactId, McpExecutionId, McpServerId};
use systemprompt::mcp::middleware::enforce_rbac_from_registry;
use systemprompt::models::artifacts::ToolResponse;
use systemprompt::models::execution::context::RequestContext;
use systemprompt::models::ExecutionMetadata;
use systemprompt_soul_extension::{CreateMemoryParams, MemoryCategory, MemoryService, MemoryType};

#[derive(Clone)]
pub struct SoulMcpServer {
    db_pool: DbPool,
    service_id: McpServerId,
}

impl SoulMcpServer {
    #[must_use]
    pub fn new(db_pool: DbPool, service_id: McpServerId) -> Self {
        Self {
            db_pool,
            service_id,
        }
    }

    fn memory_service(&self) -> Option<MemoryService> {
        let pool = self.db_pool.pool()?;
        Some(MemoryService::new(pool))
    }
}

impl ServerHandler for SoulMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: format!("Soul Memory ({})", self.service_id),
                version: env!("CARGO_PKG_VERSION").to_string(),
                icons: None,
                title: Some("Soul Memory System".to_string()),
                website_url: None,
            },
            instructions: Some(
                "Memory system for persistent agent memories. Use memory_get_context to retrieve \
                 memories for context injection, memory_store to save new memories, memory_search \
                 to find specific memories, and memory_forget to remove memories."
                    .to_string(),
            ),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParams,
        _ctx: RmcpRequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        tracing::info!("Soul MCP server initialized");
        Ok(self.get_info())
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RmcpRequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult {
            tools: tools::list_tools(),
            next_cursor: None,
            meta: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        ctx: RmcpRequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let tool_name = request.name.to_string();
        let arguments = request.arguments.clone().unwrap_or_default();

        let auth_result = enforce_rbac_from_registry(&ctx, self.service_id.as_str()).await?;
        let authenticated_ctx = auth_result
            .expect_authenticated("soul-mcp requires OAuth but auth was not enforced")?;

        let request_context = authenticated_ctx.context.clone();
        let mcp_execution_id = McpExecutionId::generate();

        match tool_name.as_str() {
            "memory_get_context" | "memory_store" | "memory_search" | "memory_forget" => {
                let service = self
                    .memory_service()
                    .ok_or_else(|| McpError::internal_error("Database not available", None))?;

                match tool_name.as_str() {
                    "memory_get_context" => {
                        self.handle_get_context(
                            arguments,
                            &service,
                            &request_context,
                            &mcp_execution_id,
                        )
                        .await
                    }
                    "memory_store" => {
                        self.handle_store(
                            arguments,
                            &service,
                            &request_context,
                            &mcp_execution_id,
                        )
                        .await
                    }
                    "memory_search" => {
                        self.handle_search(
                            arguments,
                            &service,
                            &request_context,
                            &mcp_execution_id,
                        )
                        .await
                    }
                    "memory_forget" => {
                        self.handle_forget(
                            arguments,
                            &service,
                            &request_context,
                            &mcp_execution_id,
                        )
                        .await
                    }
                    _ => unreachable!(),
                }
            }
            _ => Err(McpError::invalid_params(
                format!("Unknown tool: '{tool_name}'"),
                None,
            )),
        }
    }
}

impl SoulMcpServer {
    async fn handle_get_context(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        service: &MemoryService,
        ctx: &RequestContext,
        execution_id: &McpExecutionId,
    ) -> Result<CallToolResult, McpError> {
        let input: GetContextInput =
            serde_json::from_value(serde_json::Value::Object(arguments))
                .map_err(|e| McpError::invalid_params(format!("Invalid input: {e}"), None))?;

        let input_memory_types = input.memory_types.clone();
        let input_subject = input.subject.clone();

        let memory_types: Option<Vec<MemoryType>> = input_memory_types
            .as_ref()
            .map(|types| types.iter().filter_map(|t| t.parse().ok()).collect());

        let context_string = service
            .build_context_string(
                memory_types.as_deref(),
                input_subject.as_deref(),
                Some(input.max_items),
            )
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to get context: {e}"), None))?;

        let metadata = ExecutionMetadata::with_request(ctx).with_tool("memory_get_context");

        let artifact = serde_json::json!({
            "artifact_type": "text",
            "title": "Memory Context",
            "data": {
                "context": context_string,
                "max_items": input.max_items,
                "memory_types": input_memory_types,
                "subject": input_subject
            }
        });

        let response = ToolResponse::new(
            ArtifactId::generate(),
            execution_id.clone(),
            artifact,
            metadata.clone(),
        );

        Ok(CallToolResult {
            content: vec![Content::text(context_string)],
            is_error: Some(false),
            meta: metadata.to_meta(),
            structured_content: response.to_json().ok(),
        })
    }

    async fn handle_store(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        service: &MemoryService,
        ctx: &RequestContext,
        execution_id: &McpExecutionId,
    ) -> Result<CallToolResult, McpError> {
        let input: StoreMemoryInput = serde_json::from_value(serde_json::Value::Object(arguments))
            .map_err(|e| McpError::invalid_params(format!("Invalid input: {e}"), None))?;

        let memory_type: MemoryType = input
            .memory_type
            .parse()
            .map_err(|e: String| McpError::invalid_params(e, None))?;

        let category: MemoryCategory = input
            .category
            .parse()
            .map_err(|e: String| McpError::invalid_params(e, None))?;

        let mut params =
            CreateMemoryParams::new(memory_type, category, &input.subject, &input.content);

        if let Some(ctx_text) = input.context_text {
            params = params.with_context_text(ctx_text);
        }
        if let Some(priority) = input.priority {
            params = params.with_priority(priority);
        }
        if let Some(tags) = input.tags {
            params = params.with_tags(tags);
        }

        let memory = service
            .store(&params)
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to store memory: {e}"), None))?;

        let metadata = ExecutionMetadata::with_request(ctx).with_tool("memory_store");

        let memory_data = serde_json::json!({
            "id": memory.id,
            "memory_type": memory.memory_type,
            "category": memory.category,
            "subject": memory.subject,
            "created_at": memory.created_at,
        });

        let artifact = serde_json::json!({
            "artifact_type": "text",
            "title": "Memory Stored",
            "data": memory_data
        });

        let response = ToolResponse::new(
            ArtifactId::generate(),
            execution_id.clone(),
            artifact,
            metadata.clone(),
        );

        Ok(CallToolResult {
            content: vec![Content::text(format!(
                "Memory stored successfully.\n\n{}",
                serde_json::to_string_pretty(&memory_data).unwrap_or_default()
            ))],
            is_error: Some(false),
            meta: metadata.to_meta(),
            structured_content: response.to_json().ok(),
        })
    }

    async fn handle_search(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        service: &MemoryService,
        ctx: &RequestContext,
        execution_id: &McpExecutionId,
    ) -> Result<CallToolResult, McpError> {
        let input: SearchInput = serde_json::from_value(serde_json::Value::Object(arguments))
            .map_err(|e| McpError::invalid_params(format!("Invalid input: {e}"), None))?;

        let memories = service
            .search(
                &input.query,
                input.memory_type.as_deref(),
                input.category.as_deref(),
                Some(input.limit),
            )
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to search: {e}"), None))?;

        let metadata = ExecutionMetadata::with_request(ctx).with_tool("memory_search");

        if memories.is_empty() {
            let artifact = serde_json::json!({
                "artifact_type": "list",
                "title": "Memory Search Results",
                "data": {
                    "query": input.query,
                    "count": 0,
                    "memories": []
                }
            });

            let response = ToolResponse::new(
                ArtifactId::generate(),
                execution_id.clone(),
                artifact,
                metadata.clone(),
            );

            return Ok(CallToolResult {
                content: vec![Content::text("No memories found matching your query.")],
                is_error: Some(false),
                meta: metadata.to_meta(),
                structured_content: response.to_json().ok(),
            });
        }

        let text = memories
            .iter()
            .map(|m| {
                format!(
                    "- [{}] {} ({}): {}",
                    m.memory_type, m.subject, m.category, m.content
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let memory_data: Vec<serde_json::Value> = memories
            .iter()
            .map(|m| {
                serde_json::json!({
                    "id": m.id,
                    "memory_type": m.memory_type,
                    "category": m.category,
                    "subject": m.subject,
                    "content": m.content
                })
            })
            .collect();

        let artifact = serde_json::json!({
            "artifact_type": "list",
            "title": "Memory Search Results",
            "data": {
                "query": input.query,
                "count": memories.len(),
                "memories": memory_data
            }
        });

        let response = ToolResponse::new(
            ArtifactId::generate(),
            execution_id.clone(),
            artifact,
            metadata.clone(),
        );

        Ok(CallToolResult {
            content: vec![Content::text(format!(
                "Found {} memories:\n\n{}",
                memories.len(),
                text
            ))],
            is_error: Some(false),
            meta: metadata.to_meta(),
            structured_content: response.to_json().ok(),
        })
    }

    async fn handle_forget(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        service: &MemoryService,
        ctx: &RequestContext,
        execution_id: &McpExecutionId,
    ) -> Result<CallToolResult, McpError> {
        let input: ForgetInput = serde_json::from_value(serde_json::Value::Object(arguments))
            .map_err(|e| McpError::invalid_params(format!("Invalid input: {e}"), None))?;

        let forgotten = service
            .forget(&input.id)
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to forget: {e}"), None))?;

        let metadata = ExecutionMetadata::with_request(ctx).with_tool("memory_forget");

        if forgotten {
            let artifact = serde_json::json!({
                "artifact_type": "text",
                "title": "Memory Forgotten",
                "data": {
                    "id": input.id,
                    "status": "forgotten"
                }
            });

            let response = ToolResponse::new(
                ArtifactId::generate(),
                execution_id.clone(),
                artifact,
                metadata.clone(),
            );

            Ok(CallToolResult {
                content: vec![Content::text(format!(
                    "Memory '{}' has been forgotten.",
                    input.id
                ))],
                is_error: Some(false),
                meta: metadata.to_meta(),
                structured_content: response.to_json().ok(),
            })
        } else {
            let artifact = serde_json::json!({
                "artifact_type": "text",
                "title": "Memory Not Found",
                "data": {
                    "id": input.id,
                    "status": "not_found"
                }
            });

            let response = ToolResponse::new(
                ArtifactId::generate(),
                execution_id.clone(),
                artifact,
                metadata.clone(),
            );

            Ok(CallToolResult {
                content: vec![Content::text(format!(
                    "Memory '{}' was not found or already forgotten.",
                    input.id
                ))],
                is_error: Some(true),
                meta: metadata.to_meta(),
                structured_content: response.to_json().ok(),
            })
        }
    }
}
