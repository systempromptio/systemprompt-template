//! The knowledge-bank MCP server: struct construction and rmcp
//! `ServerHandler` surface (info, tool listing, call dispatch).
//!
//! Per-call logic (RBAC, auditing, retrieval, artifact conversion) lives in
//! the `tool` submodule. The server holds its retrieval backend as a
//! `dyn KnowledgeRetriever`, so swapping the stub for the Bedrock
//! implementation is a one-line change in `main`.

#[doc(hidden)]
pub mod tool;

use crate::error::KnowledgeBankError;
use crate::retriever::KnowledgeRetriever;
use crate::tools::{self, SERVER_NAME};
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, Implementation, InitializeRequestParams,
    InitializeResult, ListToolsResult, PaginatedRequestParams, ProtocolVersion, ServerCapabilities,
    ServerInfo,
};
use rmcp::service::{MaybeSendFuture, RequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler};
use std::future::Future;
use std::sync::Arc;
use systemprompt::database::DbPool;
use systemprompt::identifiers::McpServerId;
use systemprompt::mcp::repository::ToolUsageRepository;
use systemprompt::mcp::{
    McpArtifactRepository, McpToolExecutor, WEBSITE_URL, build_tool_list_result,
    client_profile_from_peer,
};
use systemprompt::security::authz::SharedAuthzHook;
use systemprompt_mcp_shared::record_mcp_access;
use tool::{authenticate_tool_request, dispatch_tool};

#[derive(Clone)]
pub struct KnowledgeBankServer {
    service_id: McpServerId,
    db_pool: DbPool,
    executor: McpToolExecutor,
    authz_hook: SharedAuthzHook,
    retriever: Arc<dyn KnowledgeRetriever>,
}

impl std::fmt::Debug for KnowledgeBankServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KnowledgeBankServer")
            .field("service_id", &self.service_id)
            .finish_non_exhaustive()
    }
}

impl KnowledgeBankServer {
    pub fn new(
        db_pool: DbPool,
        service_id: McpServerId,
        authz_hook: SharedAuthzHook,
        retriever: Arc<dyn KnowledgeRetriever>,
    ) -> Result<Self, KnowledgeBankError> {
        let tool_usage_repo = Arc::new(
            ToolUsageRepository::new(&db_pool)
                .map_err(|e| KnowledgeBankError::Internal(e.to_string()))?,
        );
        let artifact_repo = Arc::new(
            McpArtifactRepository::new(&db_pool)
                .map_err(|e| KnowledgeBankError::Internal(e.to_string()))?,
        );
        let executor = McpToolExecutor::new(tool_usage_repo, artifact_repo, SERVER_NAME);

        Ok(Self {
            service_id,
            db_pool,
            executor,
            authz_hook,
            retriever,
        })
    }
}

impl ServerHandler for KnowledgeBankServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_protocol_version(ProtocolVersion::V_2025_06_18)
            .with_server_info(
                Implementation::new(
                    format!("Knowledge Bank ({})", self.service_id),
                    env!("CARGO_PKG_VERSION"),
                )
                .with_title("Astound Knowledge Bank")
                .with_website_url(WEBSITE_URL),
            )
            .with_instructions(
                "Search the enterprise knowledge bank with 'search_knowledge' and discover \
                 searchable sources with 'list_knowledge_sources'.",
            )
    }

    fn initialize(
        &self,
        _request: InitializeRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<InitializeResult, McpError>> + MaybeSendFuture + '_ {
        tracing::info!("knowledge-bank MCP server initialized");
        std::future::ready(Ok(self.get_info()))
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, McpError>> + MaybeSendFuture + '_ {
        std::future::ready(Ok(build_tool_list_result(tools::list_tools())))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, McpError> {
        let tool_name = request.name.to_string();
        let server_name = self.service_id.to_string();

        let request_context = authenticate_tool_request(
            &self.db_pool,
            &tool_name,
            self.service_id.as_str(),
            &ctx,
            &self.authz_hook,
        )
        .await?;

        record_mcp_access(
            &self.db_pool,
            request_context.user_id(),
            &server_name,
            &tool_name,
            "used",
        )
        .await;

        let client = client_profile_from_peer(&ctx);
        dispatch_tool(
            &tool::Dispatch {
                executor: &self.executor,
                request: &request,
                request_context: &request_context,
                client: &client,
                retriever: self.retriever.as_ref(),
            },
            &tool_name,
        )
        .await
        .map(Into::into)
    }
}
