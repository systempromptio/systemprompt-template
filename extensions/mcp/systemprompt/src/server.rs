use crate::cli;
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
use systemprompt_mcp_shared::{record_mcp_access, record_mcp_access_rejected};

const ARTIFACT_VIEWER_TEMPLATE: &str = include_str!("../templates/artifact-viewer.html");

#[derive(Clone, Debug)]
pub struct SystempromptServer {
    service_id: McpServerId,
    db_pool: DbPool,
    executor: McpToolExecutor,
}

impl SystempromptServer {
    pub fn new(db_pool: DbPool, service_id: McpServerId) -> Result<Self, crate::error::SystempromptToolError> {
        let tool_usage_repo = Arc::new(ToolUsageRepository::new(&db_pool).map_err(|e| crate::error::SystempromptToolError::Internal(e.to_string()))?);
        let artifact_repo = Arc::new(McpArtifactRepository::new(&db_pool).map_err(|e| crate::error::SystempromptToolError::Internal(e.to_string()))?);
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
                        .unwrap_or_else(|e| {
                            tracing::warn!(error = %e, "Failed to pretty-print CLI result data, falling back to Display");
                            cmd_result.data.to_string()
                        });
                    let text_artifact = TextArtifact::new(&content, ctx);
                    CliArtifact::text(text_artifact)
                }
            }
        } else {
            let text_artifact = TextArtifact::new(&output.stdout, ctx).with_title("Command Output");
            CliArtifact::text(text_artifact)
        };

        Ok((artifact, summary))
    }
}

async fn authenticate_tool_request(
    db_pool: &DbPool,
    server_name: &str,
    tool_name: &str,
    service_id: &str,
    ctx: &RequestContext<RoleServer>,
) -> Result<(SysRequestContext, String), McpError> {
    let rbac_result = enforce_rbac_from_registry(ctx, service_id);

    match rbac_result {
        Ok(result) => {
            match result
                .expect_authenticated("BUG: systemprompt requires OAuth but auth was not enforced")
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
                    let token = authenticated.token().to_string();
                    Ok((authenticated.context.clone(), token))
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

async fn dispatch_tool(
    executor: &McpToolExecutor,
    tool_name: &str,
    request: &CallToolRequestParams,
    request_context: &SysRequestContext,
    auth_token: &str,
) -> Result<CallToolResult, McpError> {
    match tool_name {
        "systemprompt" => {
            let handler = SystempromptToolHandler {
                auth_token: auth_token.to_string(),
            };
            executor.execute(&handler, request, request_context).await
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
        tracing::info!("systemprompt.io MCP server initialized");
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

        let (request_context, auth_token) = authenticate_tool_request(
            &self.db_pool,
            &server_name,
            &tool_name,
            self.service_id.as_str(),
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

        dispatch_tool(
            &self.executor,
            &tool_name,
            &request,
            &request_context,
            &auth_token,
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
            title: "systemprompt.io Artifact Viewer",
            description: "Interactive UI viewer for systemprompt.io artifacts. Renders playbooks, lists, \
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
