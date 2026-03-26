mod constructor;

pub use constructor::MarketplaceServer;

use crate::tools::{self, SERVER_NAME};
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Implementation, InitializeRequestParams,
    InitializeResult, ListResourcesResult, ListToolsResult, PaginatedRequestParams,
    ProtocolVersion, ReadResourceRequestParams, ReadResourceResult, ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler};
use systemprompt::mcp::build_experimental_capabilities;
use systemprompt::mcp::middleware::enforce_rbac_from_registry;
use systemprompt::mcp::{
    build_artifact_viewer_resource, create_progress_callback, read_artifact_viewer_resource,
    ArtifactViewerConfig,
};

const ARTIFACT_VIEWER_TEMPLATE: &str = include_str!("../../templates/artifact-viewer.html");

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
                format!("Foodles Skill Manager ({})", self.service_id),
                env!("CARGO_PKG_VERSION"),
            )
            .with_title("Foodles Skill Manager"),
        )
        .with_instructions(
                "Foodles Skill Manager — the source of truth for all cloud agent skills, \
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
        tracing::info!("Foodles Skill Manager initialized");
        Ok(self.get_info())
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let services = tools::ToolServices {
            db_pool: self.db_pool.clone(),
            ai_service: self.ai_service.clone(),
            skill_loader: self.skill_loader.clone(),
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

        let rbac_result = enforce_rbac_from_registry(&ctx, self.service_id.as_str()).await;

        let authenticated_ctx = match rbac_result {
            Ok(result) => {
                match result.expect_authenticated(
                    "skill-manager requires OAuth but auth was not enforced",
                ) {
                    Ok(authenticated) => {
                        let pool = self.db_pool.clone();
                        let uid = authenticated.context.user_id().to_string();
                        let srv = server_name.clone();
                        let tn = tool_name.clone();
                        tokio::spawn(async move {
                            record_mcp_access(&pool, &uid, &srv, &tn, "authenticated").await;
                        });
                        authenticated
                    }
                    Err(e) => {
                        let pool = self.db_pool.clone();
                        let srv = server_name.clone();
                        let tn = tool_name.clone();
                        let reason = e.message.to_string();
                        tokio::spawn(async move {
                            record_mcp_access_rejected(&pool, &srv, &tn, &reason).await;
                        });
                        return Err(e);
                    }
                }
            }
            Err(e) => {
                let pool = self.db_pool.clone();
                let srv = server_name.clone();
                let tn = tool_name.clone();
                let reason = format!("{e}");
                tokio::spawn(async move {
                    record_mcp_access_rejected(&pool, &srv, &tn, &reason).await;
                });
                return Err(e);
            }
        };

        let request_context = authenticated_ctx.context.clone();

        // Record successful tool execution
        {
            let pool = self.db_pool.clone();
            let uid = request_context.user_id().to_string();
            let srv = server_name.clone();
            let tn = tool_name.clone();
            tokio::spawn(async move {
                record_mcp_access(&pool, &uid, &srv, &tn, "used").await;
            });
        }

        let progress_callback = ctx
            .meta
            .get_progress_token()
            .map(|token| create_progress_callback(token.clone(), ctx.peer.clone()));

        let services = tools::ToolServices {
            db_pool: self.db_pool.clone(),
            ai_service: self.ai_service.clone(),
            skill_loader: self.skill_loader.clone(),
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

async fn record_mcp_access(
    pool: &systemprompt::database::DbPool,
    user_id: &str,
    server: &str,
    tool: &str,
    action: &str,
) {
    let Some(pg_pool) = pool.pool() else {
        tracing::warn!("No PgPool available to record MCP access event");
        return;
    };
    let description = match action {
        "authenticated" => format!("Authenticated to {server} for '{tool}'"),
        "used" => format!("Executed '{tool}' on {server}"),
        _ => format!("{action} on {server}"),
    };
    let entity_type = if action == "used" { "tool" } else { "mcp_server" };
    let entity_name = if action == "used" { tool } else { server };
    let metadata = serde_json::json!({ "tool_name": tool, "server": server });

    if let Err(e) = sqlx::query(
        r"INSERT INTO user_activity (id, user_id, category, action, entity_type, entity_name, description, metadata)
          VALUES (gen_random_uuid()::TEXT, $1, 'mcp_access', $2, $3, $4, $5, $6)",
    )
    .bind(user_id)
    .bind(action)
    .bind(entity_type)
    .bind(entity_name)
    .bind(&description)
    .bind(&metadata)
    .execute(pg_pool.as_ref())
    .await
    {
        tracing::warn!(error = %e, "Failed to record MCP access event (non-fatal)");
    }
}

async fn record_mcp_access_rejected(
    pool: &systemprompt::database::DbPool,
    server: &str,
    tool: &str,
    reason: &str,
) {
    let Some(pg_pool) = pool.pool() else {
        tracing::warn!("No PgPool available to record MCP access rejection");
        return;
    };
    let description = if reason.len() > 120 {
        format!("Access rejected on {server}: {}...", &reason[..117])
    } else {
        format!("Access rejected on {server}: {reason}")
    };
    let metadata = serde_json::json!({ "tool_name": tool, "server": server, "reason": reason });

    if let Err(e) = sqlx::query(
        r"INSERT INTO user_activity (id, user_id, category, action, entity_type, entity_name, description, metadata)
          VALUES (gen_random_uuid()::TEXT, COALESCE((SELECT id FROM users WHERE email LIKE '%@anonymous.local' LIMIT 1), (SELECT id FROM users LIMIT 1)), 'mcp_access', 'rejected', 'mcp_server', $1, $2, $3)",
    )
    .bind(server)
    .bind(&description)
    .bind(&metadata)
    .execute(pg_pool.as_ref())
    .await
    {
        tracing::warn!(error = %e, "Failed to record MCP access rejection (non-fatal)");
    }
}
