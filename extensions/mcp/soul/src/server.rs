use crate::tools::{
    self, ForgetInput, GetContextInput, SearchInput, StoreMemoryInput, SERVER_NAME,
};
use anyhow::Result;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Implementation, InitializeRequestParams,
    InitializeResult, ListResourcesResult, ListToolsResult, Meta, PaginatedRequestParams,
    ProtocolVersion, RawResource, ReadResourceRequestParams, ReadResourceResult, Resource,
    ResourceContents, ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext as RmcpRequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler};
use systemprompt::database::DbPool;
use systemprompt::identifiers::{McpExecutionId, McpServerId};
use systemprompt::mcp::middleware::enforce_rbac_from_registry;
use systemprompt::mcp::services::ui_renderer::{CspPolicy, UiMetadata, MCP_APP_MIME_TYPE};
use systemprompt::mcp::{
    build_experimental_capabilities, McpArtifactRepository, McpResponseBuilder,
};
use systemprompt::models::artifacts::{ListArtifact, ListItem, TextArtifact};
use systemprompt::models::execution::context::RequestContext;
use systemprompt_soul_extension::{CreateMemoryParams, MemoryCategory, MemoryService, MemoryType};

const ARTIFACT_VIEWER_TEMPLATE: &str = include_str!("../templates/artifact-viewer.html");

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
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_experimental_with(build_experimental_capabilities())
                .build(),
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

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RmcpRequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        let resource = Resource {
            raw: RawResource {
                uri: format!("ui://{SERVER_NAME}/artifact-viewer"),
                name: "Artifact Viewer".to_string(),
                title: Some("Soul Memory Viewer".to_string()),
                description: Some(
                    "Interactive UI viewer for Soul memory artifacts. Displays memory context, \
                     search results, and stored memories with rich formatting. Template receives \
                     artifact data dynamically via MCP Apps ui/notifications/tool-result protocol."
                        .to_string(),
                ),
                mime_type: Some(MCP_APP_MIME_TYPE.to_string()),
                #[allow(clippy::cast_possible_truncation)]
                size: Some(ARTIFACT_VIEWER_TEMPLATE.len() as u32),
                icons: None,
                meta: None,
            },
            annotations: None,
        };

        Ok(ListResourcesResult {
            resources: vec![resource],
            next_cursor: None,
            meta: None,
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _ctx: RmcpRequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        let uri = &request.uri;
        let expected_uri = format!("ui://{SERVER_NAME}/artifact-viewer");

        if uri != &expected_uri {
            return Err(McpError::invalid_params(
                format!("Unknown resource URI: {uri}. Expected: {expected_uri}"),
                None,
            ));
        }

        let ui_meta = UiMetadata::for_static_template(SERVER_NAME)
            .with_csp(CspPolicy::strict())
            .with_prefers_border(true);

        let resource_meta = ui_meta.to_resource_meta();
        let meta = Meta(resource_meta.to_meta_map());

        let contents = ResourceContents::TextResourceContents {
            uri: uri.clone(),
            mime_type: Some(MCP_APP_MIME_TYPE.to_string()),
            text: ARTIFACT_VIEWER_TEMPLATE.to_string(),
            meta: Some(meta),
        };

        Ok(ReadResourceResult {
            contents: vec![contents],
        })
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

        let artifact = TextArtifact::new(&context_string, ctx).with_title("Memory Context");

        McpResponseBuilder::new(artifact, "memory_get_context", ctx, execution_id)
            .build(&context_string)
            .map_err(|e| McpError::internal_error(format!("Failed to build response: {e}"), None))
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

        let summary = format!(
            "Memory stored successfully!\nID: {}\nType: {}\nCategory: {}\nSubject: {}",
            memory.id, memory.memory_type, memory.category, memory.subject
        );

        let artifact = TextArtifact::new(&summary, ctx).with_title("Memory Stored");

        let artifact_repo = McpArtifactRepository::new(&self.db_pool).map_err(|e| {
            McpError::internal_error(format!("Failed to create artifact repository: {e}"), None)
        })?;

        McpResponseBuilder::new(artifact, "memory_store", ctx, execution_id)
            .build_and_persist(
                summary.clone(),
                &artifact_repo,
                "text",
                Some("Memory Stored".to_string()),
            )
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to build response: {e}"), None))
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

        if memories.is_empty() {
            let artifact = ListArtifact::new(ctx);

            return McpResponseBuilder::new(artifact, "memory_search", ctx, execution_id)
                .build("No memories found matching your query.")
                .map_err(|e| {
                    McpError::internal_error(format!("Failed to build response: {e}"), None)
                });
        }

        let items: Vec<ListItem> = memories
            .iter()
            .map(|m| {
                ListItem::new(&m.subject, &m.content, format!("memory://{}", m.id))
                    .with_id(m.id.to_string())
                    .with_category(&m.category)
            })
            .collect();

        let count = items.len();
        let artifact = ListArtifact::new(ctx).with_items(items);

        let summary = format!("Found {} memories matching '{}'", count, input.query);

        McpResponseBuilder::new(artifact, "memory_search", ctx, execution_id)
            .build(summary)
            .map_err(|e| McpError::internal_error(format!("Failed to build response: {e}"), None))
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

        if forgotten {
            let summary = format!("Memory '{}' has been forgotten.", input.id);

            let artifact = TextArtifact::new(&summary, ctx).with_title("Memory Forgotten");

            let artifact_repo = McpArtifactRepository::new(&self.db_pool).map_err(|e| {
                McpError::internal_error(format!("Failed to create artifact repository: {e}"), None)
            })?;

            McpResponseBuilder::new(artifact, "memory_forget", ctx, execution_id)
                .build_and_persist(
                    summary.clone(),
                    &artifact_repo,
                    "text",
                    Some("Memory Forgotten".to_string()),
                )
                .await
                .map_err(|e| {
                    McpError::internal_error(format!("Failed to build response: {e}"), None)
                })
        } else {
            // Memory not found - return error
            Ok(McpResponseBuilder::<()>::build_error(format!(
                "Memory '{}' was not found or already forgotten.",
                input.id
            )))
        }
    }
}
