use crate::tools;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Implementation, InitializeRequestParams,
    InitializeResult, ListToolsResult, PaginatedRequestParams, ProtocolVersion, ServerCapabilities,
    ServerInfo,
};
use rmcp::service::{RequestContext as RmcpRequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler};
use systemprompt::database::DbPool;
use systemprompt::identifiers::{McpExecutionId, McpServerId};
use systemprompt::mcp::middleware::enforce_rbac_from_registry;
use systemprompt_soul_extension::MemoryService;

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

    pub(crate) fn memory_service(&self) -> Option<MemoryService> {
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
                        self.handle_store(arguments, &service, &request_context, &mcp_execution_id)
                            .await
                    }
                    "memory_search" => {
                        self.handle_search(arguments, &service, &request_context, &mcp_execution_id)
                            .await
                    }
                    "memory_forget" => {
                        self.handle_forget(arguments, &service, &request_context, &mcp_execution_id)
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
