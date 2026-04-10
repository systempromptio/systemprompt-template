mod constructor;

pub use constructor::MarketplaceServer;

use std::sync::Arc;

use crate::tools::{self, SERVER_NAME};
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Implementation, InitializeRequestParams,
    InitializeResult, ListResourcesResult, ListToolsResult, PaginatedRequestParams,
    ProtocolVersion, ReadResourceRequestParams, ReadResourceResult, ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler};
use systemprompt::database::DbPool;
use systemprompt::mcp::build_experimental_capabilities;
use systemprompt::mcp::middleware::enforce_rbac_from_registry;
use systemprompt::mcp::{
    build_artifact_viewer_resource, create_progress_callback, read_artifact_viewer_resource,
    ArtifactViewerConfig,
};
use systemprompt::models::execution::context::RequestContext as AppRequestContext;
use systemprompt_mcp_shared::{record_mcp_access, record_mcp_access_rejected};

const ARTIFACT_VIEWER_TEMPLATE: &str = include_str!("../../templates/artifact-viewer.html");

async fn authenticate_request(
    db_pool: &DbPool,
    server_name: &str,
    tool_name: &str,
    service_id: &str,
    ctx: &RequestContext<RoleServer>,
) -> Result<AppRequestContext, McpError> {
    let rbac_result = enforce_rbac_from_registry(ctx, service_id);

    match rbac_result {
        Ok(result) => {
            match result
                .expect_authenticated("skill-manager requires OAuth but auth was not enforced")
            {
                Ok(authenticated) => {
                    record_mcp_access(
                        db_pool,
                        authenticated.context.user_id().as_ref(),
                        server_name,
                        tool_name,
                        "authenticated",
                    )
                    .await;
                    Ok(authenticated.context.clone())
                }
                Err(e) => {
                    record_mcp_access_rejected(db_pool, server_name, tool_name, e.message.as_ref())
                        .await;
                    Err(e)
                }
            }
        }
        Err(e) => {
            record_mcp_access_rejected(db_pool, server_name, tool_name, &format!("{e}")).await;
            Err(e)
        }
    }
}

impl ServerHandler for MarketplaceServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_experimental_with(build_experimental_capabilities())
                .build(),
        )
        .with_protocol_version(ProtocolVersion::V_2024_11_05)
        .with_server_info(
            Implementation::new(
                format!("systemprompt.io Skill Manager ({})", self.service_id),
                env!("CARGO_PKG_VERSION"),
            )
            .with_title("systemprompt.io Skill Manager"),
        )
        .with_instructions(
                "systemprompt.io Skill Manager — the source of truth for all cloud agent skills, \
                 secrets, and sync operations. All skill and secret management for agents in the \
                 cloud MUST go through this server.\n\n\
                 ## Getting Started\n\n\
                 IMPORTANT: Before creating skills, call `list_plugins` first to see the user's \
                 plugins with their skills, agents, MCP servers, and onboarding configuration. \
                 Then call `list_skills` to check for custom skills. If the user has plugins with \
                 onboarding config, follow the marketplace_onboarding skill instructions — it uses \
                 plugin-specific interview questions tailored to each plugin's domain.\n\n\
                 ### Onboarding (when user has no custom skills)\n\n\
                 1. Call `list_plugins` to get the user's selected plugins and their onboarding config\n\
                 2. Follow the marketplace_onboarding skill — it guides a Socratic discovery interview \
                 using plugin-specific questions from the onboarding config\n\
                 3. Use `create_skill` with `target_plugin_id` to add personalized skills to the correct plugin\n\n\
                 ## Tools\n\n\
                 - `list_plugins` — List user's plugins with skills, agents, MCP servers, and onboarding config\n\
                 - `create_skill` — Create a new skill (supports optional `target_plugin_id` to add to a specific plugin)\n\
                 - `update_skill` — Update an existing skill\n\
                 - `list_skills` — List all user skills with usage counts\n\
                 - `create_agent` — Create a new agent (auto-added to user's plugin)\n\
                 - `update_agent` — Update an existing agent\n\
                 - `list_agents` — List all user agents\n\
                 - `analyze_skill` — AI-powered skill quality analysis\n\
                 - `get_secrets` / `manage_secrets` — Plugin environment variables\n\
                 - `sync_skills` — Sync skills to server\n\n\
                 ## Important Notes\n\n\
                 - This server is the single authoritative source for all cloud skill and secret \
                 operations. Any changes to agent skills or secrets must be made through these tools.\n\
                 - Created skills and agents are automatically added to the user's plugin. Use \
                 `target_plugin_id` on `create_skill` to add to a specific plugin instead of the default.\n\
                 - When creating skills, NEVER ask the user to write YAML or config. Ask \
                 plain-language questions and translate answers into skill content. Use the \
                 skill_creator interview pattern: purpose → examples → instructions → rules → \
                 tags → voice.\n\
                 - When managing secrets, always use is_secret: true for credentials. Never \
                 repeat secret values back to the user after saving.",
        )
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        tracing::info!("systemprompt.io Skill Manager initialized");
        Ok(self.get_info())
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let services = tools::ToolServices {
            db_pool: DbPool::clone(&self.db_pool),
            ai_service: Arc::clone(&self.ai_service),
            skill_loader: Arc::clone(&self.skill_loader),
            executor: self.executor.clone(),
        };
        Ok(ListToolsResult {
            tools: tools::list_tools(&services),
            next_cursor: None,
            meta: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let tool_name = request.name.to_string();
        let server_name = self.service_id.to_string();

        let request_context = authenticate_request(
            &self.db_pool,
            &server_name,
            &tool_name,
            self.service_id.as_ref(),
            &ctx,
        )
        .await?;

        record_mcp_access(
            &self.db_pool,
            request_context.user_id().as_ref(),
            &server_name,
            &tool_name,
            "used",
        )
        .await;

        let progress_callback = ctx
            .meta
            .get_progress_token()
            .map(|token| create_progress_callback(token, ctx.peer.clone()));

        let services = tools::ToolServices {
            db_pool: DbPool::clone(&self.db_pool),
            ai_service: Arc::clone(&self.ai_service),
            skill_loader: Arc::clone(&self.skill_loader),
            executor: self.executor.clone(),
        };

        tools::handle_tool_call(
            &tool_name,
            request,
            request_context,
            &services,
            progress_callback,
        )
        .await
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(build_artifact_viewer_resource(&ArtifactViewerConfig {
            server_name: SERVER_NAME,
            title: "Skill Manager Viewer",
            description: "Interactive UI viewer for Skill Manager artifacts. Displays skill data, \
                         analysis results, and sync status with rich formatting.",
            template: ARTIFACT_VIEWER_TEMPLATE,
            icons: None,
        }))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        read_artifact_viewer_resource(&request, SERVER_NAME, ARTIFACT_VIEWER_TEMPLATE)
    }
}
