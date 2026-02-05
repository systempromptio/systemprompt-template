use crate::cli;
use crate::tools::{self, CliInput, SERVER_NAME};
use anyhow::Result;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Icon, Implementation, InitializeRequestParams,
    InitializeResult, ListResourcesResult, ListToolsResult, Meta, PaginatedRequestParams,
    ProtocolVersion, RawResource, ReadResourceRequestParams, ReadResourceResult, Resource,
    ResourceContents, ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler};
use systemprompt::database::DbPool;
use systemprompt::identifiers::{McpExecutionId, McpServerId};
use systemprompt::mcp::middleware::enforce_rbac_from_registry;
use systemprompt::mcp::services::ui_renderer::{CspPolicy, UiMetadata, MCP_APP_MIME_TYPE};
use systemprompt::mcp::{
    build_experimental_capabilities, McpArtifactRepository, McpResponseBuilder, WEBSITE_URL,
};
use systemprompt::models::artifacts::{CliArtifact, CommandResultRaw, TextArtifact};
use systemprompt::models::execution::context::RequestContext as SysRequestContext;

const ARTIFACT_VIEWER_TEMPLATE: &str = include_str!("../templates/artifact-viewer.html");

#[derive(Clone)]
pub struct SystempromptServer {
    db_pool: DbPool,
    service_id: McpServerId,
}

impl SystempromptServer {
    #[must_use]
    pub fn new(db_pool: DbPool, service_id: McpServerId) -> Self {
        Self {
            db_pool,
            service_id,
        }
    }
}

impl ServerHandler for SystempromptServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_experimental_with(build_experimental_capabilities())
                .build(),
            server_info: Implementation {
                name: format!("SystemPrompt ({})", self.service_id),
                version: env!("CARGO_PKG_VERSION").to_string(),
                icons: Some(vec![
                    Icon {
                        src: format!("{WEBSITE_URL}/files/images/favicon-32x32.png"),
                        mime_type: Some("image/png".to_string()),
                        sizes: Some(vec!["32x32".to_string()]),
                    },
                    Icon {
                        src: format!("{WEBSITE_URL}/files/images/favicon-96x96.png"),
                        mime_type: Some("image/png".to_string()),
                        sizes: Some(vec!["96x96".to_string()]),
                    },
                ]),
                title: Some("SystemPrompt CLI".to_string()),
                website_url: Some(WEBSITE_URL.to_string()),
            },
            instructions: Some(
                "Execute SystemPrompt CLI commands. MANDATORY FIRST STEP: Run 'core playbooks show guide_start' \
                 before any task. Playbooks: 'core playbooks show <id>' or 'core playbooks list'. \
                 Discord: 'plugins run discord send \"message\"'. Full documentation: https://systemprompt.io/playbooks"
                    .to_string(),
            ),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        tracing::info!("SystemPrompt MCP server initialized");
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

        let auth_result = enforce_rbac_from_registry(&ctx, self.service_id.as_str())
            .await?
            .expect_authenticated("BUG: systemprompt requires OAuth but auth was not enforced")?;

        let request_context = auth_result.context.clone();
        let mcp_execution_id = McpExecutionId::generate();

        let arguments = request.arguments.clone().unwrap_or_default();

        match tool_name.as_str() {
            "systemprompt" => {
                self.handle_systemprompt_tool(
                    arguments,
                    auth_result.token(),
                    &request_context,
                    &mcp_execution_id,
                )
                .await
            }
            _ => Err(McpError::invalid_params(
                format!(
                    "Unknown tool: '{tool_name}'\n\nMANDATORY FIRST STEP: Run 'core playbooks show \
                     guide_start' before any task.\n\nUse 'systemprompt' tool with command 'core \
                     playbooks show guide_start' to get started."
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
        let resource = Resource {
            raw: RawResource {
                uri: format!("ui://{SERVER_NAME}/artifact-viewer"),
                name: "Artifact Viewer".to_string(),
                title: Some("systemprompt.io Artifact Viewer".to_string()),
                description: Some(
                    "Interactive UI viewer for systemprompt.io artifacts. Renders playbooks, lists, \
                     and text content with syntax highlighting. Template receives artifact data \
                     dynamically via MCP Apps ui/notifications/tool-result protocol."
                        .to_string(),
                ),
                mime_type: Some(MCP_APP_MIME_TYPE.to_string()),
                #[allow(clippy::cast_possible_truncation)]
                size: Some(ARTIFACT_VIEWER_TEMPLATE.len() as u32),
                icons: Some(vec![
                    Icon {
                        src: format!("{WEBSITE_URL}/files/images/favicon-32x32.png"),
                        mime_type: Some("image/png".to_string()),
                        sizes: Some(vec!["32x32".to_string()]),
                    },
                ]),
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
        _ctx: RequestContext<RoleServer>,
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

impl SystempromptServer {
    async fn handle_systemprompt_tool(
        &self,
        arguments: serde_json::Map<String, serde_json::Value>,
        auth_token: &str,
        ctx: &SysRequestContext,
        execution_id: &McpExecutionId,
    ) -> Result<CallToolResult, McpError> {
        let input: CliInput = serde_json::from_value(serde_json::Value::Object(arguments))
            .map_err(|e| {
                McpError::invalid_params(format!("Invalid input parameters: {e}"), None)
            })?;

        let output = cli::execute(&input.command, auth_token)?;

        if !output.success {
            let error_message = format!(
                "Command failed (exit code {}):\n{}",
                output.exit_code, output.stderr
            );
            return Ok(McpResponseBuilder::<()>::build_error(error_message));
        }

        let artifact_repo = McpArtifactRepository::new(&self.db_pool).map_err(|e| {
            McpError::internal_error(format!("Failed to create artifact repository: {e}"), None)
        })?;

        let (artifact, artifact_type, title) = if let Ok(cmd_result) =
            CommandResultRaw::from_json(&output.stdout)
        {
            let artifact_type = format!("{:?}", cmd_result.artifact_type).to_lowercase();
            let title = cmd_result.title.clone();

            match cmd_result.to_cli_artifact(ctx) {
                Ok(artifact) => (artifact, artifact_type, title),
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to convert CLI result to artifact, falling back to text");
                    let content = serde_json::to_string_pretty(&cmd_result.data)
                        .unwrap_or_else(|_| cmd_result.data.to_string());
                    let text_artifact = TextArtifact::new(&content, ctx);
                    (CliArtifact::text(text_artifact), "text".to_string(), title)
                }
            }
        } else {
            let text_artifact = TextArtifact::new(&output.stdout, ctx).with_title("Command Output");
            (
                CliArtifact::text(text_artifact),
                "text".to_string(),
                Some("Command Output".to_string()),
            )
        };

        McpResponseBuilder::new(artifact, SERVER_NAME, ctx, execution_id)
            .build_and_persist(output.stdout.clone(), &artifact_repo, &artifact_type, title)
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to build response: {e}"), None))
    }
}
