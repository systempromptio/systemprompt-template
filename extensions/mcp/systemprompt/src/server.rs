use crate::cli;
use crate::server_logging::{record_mcp_access, record_mcp_access_rejected};
use crate::tools::{self, CliInput, SERVER_NAME};
use async_trait::async_trait;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Icon, Implementation, InitializeRequestParams,
    InitializeResult, ListResourcesResult, ListToolsResult, PaginatedRequestParams,
    ProtocolVersion, ReadResourceRequestParams, ReadResourceResult, ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler};
use std::sync::Arc;
use systemprompt::database::DbPool;
use systemprompt::identifiers::{McpExecutionId, McpServerId};
use systemprompt::mcp::middleware::enforce_rbac_from_registry;
use systemprompt::mcp::repository::ToolUsageRepository;
use systemprompt::mcp::{
    build_artifact_viewer_resource, build_experimental_capabilities, read_artifact_viewer_resource,
    ArtifactViewerConfig, McpArtifactRepository, McpToolExecutor, McpToolHandler, WEBSITE_URL,
};
use systemprompt::models::artifacts::{CliArtifact, CommandResultRaw, TextArtifact};
use systemprompt::models::execution::context::RequestContext as SysRequestContext;

const ARTIFACT_VIEWER_TEMPLATE: &str = include_str!("../templates/artifact-viewer.html");

#[derive(Clone)]
pub struct SystempromptServer {
    service_id: McpServerId,
    db_pool: DbPool,
    executor: McpToolExecutor,
}

impl SystempromptServer {
    pub fn new(db_pool: DbPool, service_id: McpServerId) -> anyhow::Result<Self> {
        let tool_usage_repo = Arc::new(ToolUsageRepository::new(&db_pool)?);
        let artifact_repo = Arc::new(McpArtifactRepository::new(&db_pool)?);
        let executor = McpToolExecutor::new(tool_usage_repo, artifact_repo, SERVER_NAME);

        Ok(Self {
            service_id,
            db_pool,
            executor,
        })
    }
}

struct SystempromptToolHandler {
    auth_token: String,
}

#[async_trait]
impl McpToolHandler for SystempromptToolHandler {
    type Input = CliInput;
    type Output = CliArtifact;

    fn tool_name(&self) -> &'static str {
        "systemprompt"
    }

    fn description(&self) -> &'static str {
        "Execute SystemPrompt CLI commands."
    }

    async fn handle(
        &self,
        input: Self::Input,
        ctx: &SysRequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        let output = cli::execute(&input.command, &self.auth_token)?;

        if !output.success {
            return Err(McpError::internal_error(
                format!(
                    "Command failed (exit code {}):\n{}",
                    output.exit_code, output.stderr
                ),
                None,
            ));
        }

        let summary = output.stdout.clone();

        let artifact = if let Ok(cmd_result) = CommandResultRaw::from_json(&output.stdout) {
            match cmd_result.to_cli_artifact(ctx) {
                Ok(artifact) => artifact,
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to convert CLI result to artifact, falling back to text");
                    let content = serde_json::to_string_pretty(&cmd_result.data)
                        .unwrap_or_else(|_| cmd_result.data.to_string());
                    let text_artifact = TextArtifact::new(&content, ctx);
                    CliArtifact::text(text_artifact)
                }
            }
        } else {
            let text_artifact =
                TextArtifact::new(&output.stdout, ctx).with_title("Command Output");
            CliArtifact::text(text_artifact)
        };

        Ok((artifact, summary))
    }
}

impl ServerHandler for SystempromptServer {
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
                format!("SystemPrompt ({})", self.service_id),
                env!("CARGO_PKG_VERSION"),
            )
            .with_title("SystemPrompt CLI")
            .with_icons(vec![
                Icon::new(format!("{WEBSITE_URL}/files/images/favicon-32x32.png"))
                    .with_mime_type("image/png")
                    .with_sizes(vec!["32x32".to_string()]),
                Icon::new(format!("{WEBSITE_URL}/files/images/favicon-96x96.png"))
                    .with_mime_type("image/png")
                    .with_sizes(vec!["96x96".to_string()]),
            ])
            .with_website_url(WEBSITE_URL),
        )
        .with_instructions(
            format!("Execute SystemPrompt CLI commands. MANDATORY FIRST STEP: Run 'core playbooks show guide_start' \
             before any task. Playbooks: 'core playbooks show <id>' or 'core playbooks list'. \
             Discord: 'plugins run discord send \"message\"'. Full documentation: {WEBSITE_URL}/playbooks"),
        )
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        tracing::info!("Foodles MCP server initialized");
        Ok(self.get_info())
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let tool_list = tools::list_tools();
        Ok(ListToolsResult {
            tools: tool_list,
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

        let auth_result = match rbac_result {
            Ok(result) => {
                match result.expect_authenticated(
                    "BUG: systemprompt requires OAuth but auth was not enforced",
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

        let request_context = auth_result.context.clone();

        {
            let pool = self.db_pool.clone();
            let uid = request_context.user_id().to_string();
            let srv = server_name.clone();
            let tn = tool_name.clone();
            tokio::spawn(async move {
                record_mcp_access(&pool, &uid, &srv, &tn, "used").await;
            });
        }

        match tool_name.as_str() {
            "systemprompt" => {
                let handler = SystempromptToolHandler {
                    auth_token: auth_result.token().to_string(),
                };
                self.executor
                    .execute(&handler, &request, &request_context)
                    .await
            }
            _ => Err(McpError::invalid_params(
                format!(
                    "Unknown tool: '{tool_name}'\n\nMANDATORY FIRST STEP: Run 'core skills show \
                     systemprompt_cli' before any task.\n\nUse 'systemprompt' tool with command 'core \
                     skills show systemprompt_cli' to get started."
                ),
                None,
            )),
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(build_artifact_viewer_resource(&ArtifactViewerConfig {
            server_name: SERVER_NAME,
            title: "Foodles Artifact Viewer",
            description: "Interactive UI viewer for Foodles artifacts. Renders playbooks, lists, \
                         and text content with syntax highlighting. Template receives artifact data \
                         dynamically via MCP Apps ui/notifications/tool-result protocol.",
            template: ARTIFACT_VIEWER_TEMPLATE,
            icons: Some(vec![
                Icon::new(format!("{WEBSITE_URL}/files/images/favicon-32x32.png"))
                    .with_mime_type("image/png")
                    .with_sizes(vec!["32x32".to_string()]),
            ]),
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
