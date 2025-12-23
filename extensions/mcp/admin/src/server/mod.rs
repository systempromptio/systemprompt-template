pub mod constructor;
mod handlers;

pub use constructor::AdminServer;

use anyhow::Result;
use rmcp::{model::{ServerInfo, InitializeRequestParam, InitializeResult, PaginatedRequestParam, ListToolsResult, CallToolRequestParam, CallToolResult, ListPromptsResult, GetPromptRequestParam, GetPromptResult, ListResourcesResult, ReadResourceRequestParam, ReadResourceResult, ListResourceTemplatesResult}, service::RequestContext, ErrorData as McpError, RoleServer, ServerHandler};

impl ServerHandler for AdminServer {
    fn get_info(&self) -> ServerInfo {
        self.get_info()
    }

    async fn initialize(
        &self,
        request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        self.initialize(request, context).await
    }

    async fn list_tools(
        &self,
        request: Option<PaginatedRequestParam>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        self.list_tools(request, ctx).await
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        AdminServer::call_tool(self, request, ctx).await
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
        request: Option<PaginatedRequestParam>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        self.resources.list_resources(request, ctx).await
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        ctx: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        self.resources.read_resource(request, ctx).await
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
